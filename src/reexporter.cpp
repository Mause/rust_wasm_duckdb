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
