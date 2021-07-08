#include <emscripten.h>
#include "duckdb.hpp"
#include <iostream>

using namespace duckdb;

struct DatabaseData {
  DatabaseData() : database(nullptr) {}
  ~DatabaseData() {
    if (database) {
      delete database;
    }
  }

  duckdb::DuckDB *database;
};

template <class T>
T UnsafeFetch(duckdb_result *result, idx_t col, idx_t row)
{
    D_ASSERT(row < result->row_count);
    return ((T *)result->columns[col].data)[row];
}

template <class T>
T lookup(duckdb_result *result, idx_t col, idx_t row)
{
    if (col >= result->column_count)
    {
        return nullptr;
    }
    if (row >= result->row_count)
    {
        return nullptr;
    }
    if (result->columns[col].nullmask[row])
    {
        return nullptr;
    }

    return UnsafeFetch<T>(result, col, row);
}

extern "C"
{
    void *mallocy()
    {
        return calloc(1, sizeof(void *));
    }

    duckdb_date *duckdb_value_date(duckdb_result *result, idx_t col, idx_t row)
    {
        return lookup<duckdb_date *>(result, col, row);
    }

    duckdb_time *duckdb_value_time(duckdb_result *result, idx_t col, idx_t row)
    {
        return lookup<duckdb_time *>(result, col, row);
    }

    duckdb_interval *duckdb_value_interval(duckdb_result *result, idx_t col, idx_t row)
    {
        return lookup<duckdb_interval *>(result, col, row);
    }

    duckdb_hugeint *duckdb_value_hugeint(duckdb_result *result, idx_t col, idx_t row)
    {
        return lookup<duckdb_hugeint *>(result, col, row);
    }

    duckdb_timestamp *duckdb_value_timestamp(duckdb_result *result, idx_t col, idx_t row)
    {
        return lookup<duckdb_timestamp *>(result, col, row);
    }

    void ext_duckdb_close(duckdb_database *database) {
        if (*database) {
            auto wrapper = (DatabaseData *)*database;
            delete wrapper;
            *database = nullptr;
        }
    }

    duckdb_connection create_connection(DatabaseData* db) {
        return (duckdb_connection) new duckdb::Connection(*db->database);
    }
}
