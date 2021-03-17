#include <emscripten.h>
#include "duckdb.hpp"
#include <iostream>

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
            out->column_count = result->types.size();
            out->row_count = result->collection.Count();
            // out->columns = (duckdb_column *)calloc(sizeof(duckdb_column), out->column_count);
            // std::cerr << out->columns << std::endl;
        }
        else
        {
            out->error_message = strdup(result->error.c_str());
        }

        return out;
    }
}
