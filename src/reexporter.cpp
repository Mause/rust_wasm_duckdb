#include <emscripten.h>
#include "duckdb.hpp"
#include <iostream>

using namespace duckdb;

static duckdb_type ConvertCPPTypeToC(LogicalType type);
static idx_t GetCTypeSize(duckdb_type type);
struct DatabaseData
{
    DatabaseData() : database(nullptr)
    {
    }
    ~DatabaseData()
    {
        if (database)
        {
            delete database;
        }
    }

    duckdb::DuckDB *database;
};

template <class T>
void WriteData(duckdb_result *out, ChunkCollection &source, idx_t col)
{
    idx_t row = 0;
    auto target = (T *)out->columns[col].data;
    for (auto &chunk : source.Chunks())
    {
        auto source = FlatVector::GetData<T>(chunk->data[col]);
        auto &mask = FlatVector::Validity(chunk->data[col]);

        for (idx_t k = 0; k < chunk->size(); k++, row++)
        {
            if (!mask.RowIsValid(k))
            {
                continue;
            }
            target[row] = source[k];
        }
    }
}

static duckdb_state duckdb_translate_result(MaterializedQueryResult *result, duckdb_result *out)
{
    D_ASSERT(result);
    if (!out)
    {
        // no result to write to, only return the status
        return result->success ? DuckDBSuccess : DuckDBError;
    }
    out->error_message = nullptr;
    if (!result->success)
    {
        // write the error message
        out->error_message = strdup(result->error.c_str());
        return DuckDBError;
    }
    // copy the data
    // first write the meta data
    out->column_count = result->types.size();
    out->row_count = result->collection.Count();
    out->columns = (duckdb_column *)malloc(sizeof(duckdb_column) * out->column_count);
    if (!out->columns)
    {
        return DuckDBError;
    }
    // zero initialize the columns (so we can cleanly delete it in case a malloc fails)
    memset(out->columns, 0, sizeof(duckdb_column) * out->column_count);
    for (idx_t i = 0; i < out->column_count; i++)
    {
        out->columns[i].type = ConvertCPPTypeToC(result->types[i]);
        out->columns[i].name = strdup(result->names[i].c_str());
        out->columns[i].nullmask = (bool *)malloc(sizeof(bool) * out->row_count);
        out->columns[i].data = malloc(GetCTypeSize(out->columns[i].type) * out->row_count);
        if (!out->columns[i].nullmask || !out->columns[i].name || !out->columns[i].data)
        {
            // malloc failure
            return DuckDBError;
        }
        // memset data to 0 for VARCHAR columns for safe deletion later
        if (result->types[i].InternalType() == PhysicalType::VARCHAR)
        {
            memset(out->columns[i].data, 0, GetCTypeSize(out->columns[i].type) * out->row_count);
        }
    }
    // now write the data
    for (idx_t col = 0; col < out->column_count; col++)
    {
        // first set the nullmask
        idx_t row = 0;
        for (auto &chunk : result->collection.Chunks())
        {
            for (idx_t k = 0; k < chunk->size(); k++)
            {
                out->columns[col].nullmask[row++] = FlatVector::IsNull(chunk->data[col], k);
            }
        }
        // then write the data
        switch (result->types[col].id())
        {
        case LogicalTypeId::BOOLEAN:
            WriteData<bool>(out, result->collection, col);
            break;
        case LogicalTypeId::TINYINT:
            WriteData<int8_t>(out, result->collection, col);
            break;
        case LogicalTypeId::SMALLINT:
            WriteData<int16_t>(out, result->collection, col);
            break;
        case LogicalTypeId::INTEGER:
            WriteData<int32_t>(out, result->collection, col);
            break;
        case LogicalTypeId::BIGINT:
            WriteData<int64_t>(out, result->collection, col);
            break;
        case LogicalTypeId::FLOAT:
            WriteData<float>(out, result->collection, col);
            break;
        case LogicalTypeId::DOUBLE:
            WriteData<double>(out, result->collection, col);
            break;
        case LogicalTypeId::VARCHAR:
        {
            idx_t row = 0;
            auto target = (const char **)out->columns[col].data;
            for (auto &chunk : result->collection.Chunks())
            {
                auto source = FlatVector::GetData<string_t>(chunk->data[col]);
                for (idx_t k = 0; k < chunk->size(); k++)
                {
                    if (!FlatVector::IsNull(chunk->data[col], k))
                    {
                        target[row] = (char *)malloc(source[k].GetSize() + 1);
                        assert(target[row]);
                        memcpy((void *)target[row], source[k].GetDataUnsafe(), source[k].GetSize());
                        auto write_arr = (char *)target[row];
                        write_arr[source[k].GetSize()] = '\0';
                    }
                    row++;
                }
            }
            break;
        }
        case LogicalTypeId::DATE:
        {
            idx_t row = 0;
            auto target = (duckdb_date *)out->columns[col].data;
            for (auto &chunk : result->collection.Chunks())
            {
                auto source = FlatVector::GetData<date_t>(chunk->data[col]);
                for (idx_t k = 0; k < chunk->size(); k++)
                {
                    if (!FlatVector::IsNull(chunk->data[col], k))
                    {
                        int32_t year, month, day;
                        Date::Convert(source[k], year, month, day);
                        target[row].year = year;
                        target[row].month = month;
                        target[row].day = day;
                    }
                    row++;
                }
            }
            break;
        }
        case LogicalTypeId::TIME:
        {
            idx_t row = 0;
            auto target = (duckdb_time *)out->columns[col].data;
            for (auto &chunk : result->collection.Chunks())
            {
                auto source = FlatVector::GetData<dtime_t>(chunk->data[col]);
                for (idx_t k = 0; k < chunk->size(); k++)
                {
                    if (!FlatVector::IsNull(chunk->data[col], k))
                    {
                        int32_t hour, min, sec, micros;
                        Time::Convert(source[k], hour, min, sec, micros);
                        target[row].hour = hour;
                        target[row].min = min;
                        target[row].sec = sec;
                        target[row].micros = micros;
                    }
                    row++;
                }
            }
            break;
        }
        case LogicalTypeId::TIMESTAMP:
        {
            idx_t row = 0;
            auto target = (duckdb_timestamp *)out->columns[col].data;
            for (auto &chunk : result->collection.Chunks())
            {
                auto source = FlatVector::GetData<timestamp_t>(chunk->data[col]);
                for (idx_t k = 0; k < chunk->size(); k++)
                {
                    if (!FlatVector::IsNull(chunk->data[col], k))
                    {
                        date_t date;
                        dtime_t time;
                        Timestamp::Convert(source[k], date, time);

                        int32_t year, month, day;
                        Date::Convert(date, year, month, day);

                        int32_t hour, min, sec, micros;
                        Time::Convert(time, hour, min, sec, micros);

                        target[row].date.year = year;
                        target[row].date.month = month;
                        target[row].date.day = day;
                        target[row].time.hour = hour;
                        target[row].time.min = min;
                        target[row].time.sec = sec;
                        target[row].time.micros = micros;
                    }
                    row++;
                }
            }
            break;
        }
        case LogicalTypeId::HUGEINT:
        {
            idx_t row = 0;
            auto target = (duckdb_hugeint *)out->columns[col].data;
            for (auto &chunk : result->collection.Chunks())
            {
                auto source = FlatVector::GetData<hugeint_t>(chunk->data[col]);
                for (idx_t k = 0; k < chunk->size(); k++)
                {
                    if (!FlatVector::IsNull(chunk->data[col], k))
                    {
                        target[row].lower = source[k].lower;
                        target[row].upper = source[k].upper;
                    }
                    row++;
                }
            }
            break;
        }
        case LogicalTypeId::INTERVAL:
        {
            idx_t row = 0;
            auto target = (duckdb_interval *)out->columns[col].data;
            for (auto &chunk : result->collection.Chunks())
            {
                auto source = FlatVector::GetData<interval_t>(chunk->data[col]);
                for (idx_t k = 0; k < chunk->size(); k++)
                {
                    if (!FlatVector::IsNull(chunk->data[col], k))
                    {
                        target[row].days = source[k].days;
                        target[row].months = source[k].months;
                        target[row].micros = source[k].micros;
                    }
                    row++;
                }
            }
            break;
        }
        default:
            // unsupported type for C API
            D_ASSERT(0);
            return DuckDBError;
        }
    }
    return DuckDBSuccess;
}

duckdb_type ConvertCPPTypeToC(LogicalType sql_type)
{
    switch (sql_type.id())
    {
    case LogicalTypeId::BOOLEAN:
        return DUCKDB_TYPE_BOOLEAN;
    case LogicalTypeId::TINYINT:
        return DUCKDB_TYPE_TINYINT;
    case LogicalTypeId::SMALLINT:
        return DUCKDB_TYPE_SMALLINT;
    case LogicalTypeId::INTEGER:
        return DUCKDB_TYPE_INTEGER;
    case LogicalTypeId::BIGINT:
        return DUCKDB_TYPE_BIGINT;
    case LogicalTypeId::HUGEINT:
        return DUCKDB_TYPE_HUGEINT;
    case LogicalTypeId::FLOAT:
        return DUCKDB_TYPE_FLOAT;
    case LogicalTypeId::DOUBLE:
        return DUCKDB_TYPE_DOUBLE;
    case LogicalTypeId::TIMESTAMP:
        return DUCKDB_TYPE_TIMESTAMP;
    case LogicalTypeId::DATE:
        return DUCKDB_TYPE_DATE;
    case LogicalTypeId::TIME:
        return DUCKDB_TYPE_TIME;
    case LogicalTypeId::VARCHAR:
        return DUCKDB_TYPE_VARCHAR;
    case LogicalTypeId::BLOB:
        return DUCKDB_TYPE_BLOB;
    case LogicalTypeId::INTERVAL:
        return DUCKDB_TYPE_INTERVAL;
    default:
        return DUCKDB_TYPE_INVALID;
    }
}

idx_t GetCTypeSize(duckdb_type type)
{
    switch (type)
    {
    case DUCKDB_TYPE_BOOLEAN:
        return sizeof(bool);
    case DUCKDB_TYPE_TINYINT:
        return sizeof(int8_t);
    case DUCKDB_TYPE_SMALLINT:
        return sizeof(int16_t);
    case DUCKDB_TYPE_INTEGER:
        return sizeof(int32_t);
    case DUCKDB_TYPE_BIGINT:
        return sizeof(int64_t);
    case DUCKDB_TYPE_HUGEINT:
        return sizeof(duckdb_hugeint);
    case DUCKDB_TYPE_FLOAT:
        return sizeof(float);
    case DUCKDB_TYPE_DOUBLE:
        return sizeof(double);
    case DUCKDB_TYPE_DATE:
        return sizeof(duckdb_date);
    case DUCKDB_TYPE_TIME:
        return sizeof(duckdb_time);
    case DUCKDB_TYPE_TIMESTAMP:
        return sizeof(duckdb_timestamp);
    case DUCKDB_TYPE_VARCHAR:
        return sizeof(const char *);
    case DUCKDB_TYPE_BLOB:
        return sizeof(duckdb_blob);
    case DUCKDB_TYPE_INTERVAL:
        return sizeof(duckdb_interval);
    default:
        std::cout << "Unsupported type: " << type << std::endl;
        // unsupported type
        D_ASSERT(0);
        return sizeof(const char *);
    }
}

template <class T>
T UnsafeFetch(duckdb_result *result, idx_t col, idx_t row) {
	D_ASSERT(row < result->row_count);
	return ((T *)result->columns[col].data)[row];
}

static Value GetCValue(duckdb_result *result, idx_t col, idx_t row)
{
    if (col >= result->column_count)
    {
        return Value();
    }
    if (row >= result->row_count)
    {
        return Value();
    }
    if (result->columns[col].nullmask[row])
    {
        return Value();
    }
    switch (result->columns[col].type)
    {
    case DUCKDB_TYPE_DATE:
    {
        auto date = UnsafeFetch<duckdb_date>(result, col, row);
        return Value::DATE(date.year, date.month, date.day);
    }
    default:
        return Value();
    }
}

extern "C"
{
    void* mallocy() {
        return calloc(1, sizeof(void*));
    }

    duckdb_date *duckdb_value_date(duckdb_result *result, idx_t col, idx_t row)
    {
        return &((duckdb_date *)result->columns[col].data)[row];
    }

    duckdb_time *duckdb_value_time(duckdb_result *result, idx_t col, idx_t row)
    {
        return &((duckdb_time *)result->columns[col].data)[row];
    }

    duckdb_timestamp *duckdb_value_timestamp(duckdb_result *result, idx_t col, idx_t row)
    {
        return &((duckdb_timestamp *)result->columns[col].data)[row];
    }

    void ext_duckdb_close(duckdb_database *database) {
        if (*database) {
            auto wrapper = (DatabaseData *)database;
            delete wrapper;
            *database = nullptr;
        }
    }

    duckdb_state query(DatabaseData *db, const char *query, duckdb_result *out)
    {
        duckdb::Connection con(*db->database);

        auto res = con.Query(query);

        return duckdb_translate_result(res.get(), out);
    }
}
