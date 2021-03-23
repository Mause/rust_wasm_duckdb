#include <emscripten.h>
#include "duckdb.hpp"
#include <iostream>

static duckdb_state duckdb_translate_result(void *result, duckdb_result *out);

extern "C"
{
    duckdb_result *query(const char *query)
    {
        duckdb::DuckDB db(nullptr);
        duckdb::Connection con(db);
        auto result = con.Query(query);

        duckdb_result *out = (duckdb_result *)malloc(sizeof(duckdb_result *));

        out->error_message = nullptr;
        if (result->success)
        {
            duckdb_translate_result(result.get(), out);
        }
        else
        {
            out->error_message = strdup(result->error.c_str());
        }

        return out;
    }
}
