#include <emscripten.h>
#include "duckdb.hpp"
#include <iostream>

extern "C"
{
    duckdb_result *query(const char *query)
    {
        duckdb::DuckDB db(nullptr);
        duckdb::Connection con(db);

        duckdb_result *out = (duckdb_result *)malloc(sizeof(duckdb_result *));

        duckdb_query(con, query, out);

        return out;
    }
}
