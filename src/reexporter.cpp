#include <emscripten.h>
#include "duckdb.hpp"
#include <iostream>

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

    duckdb_interval *duckdb_value_interval(duckdb_result *result, idx_t col, idx_t row)
    {
        return &((duckdb_interval *)result->columns[col].data)[row];
    }

    duckdb_hugeint *duckdb_value_hugeint(duckdb_result *result, idx_t col, idx_t row)
    {
        return &((duckdb_hugeint *)result->columns[col].data)[row];
    }

    duckdb_timestamp *duckdb_value_timestamp(duckdb_result *result, idx_t col, idx_t row)
    {
        return &((duckdb_timestamp *)result->columns[col].data)[row];
    }
}
