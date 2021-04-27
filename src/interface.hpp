#include "../target/duckdb.h"

class DuckDB {
public:
// 	DUCKDB_API explicit DuckDB(const char *path = nullptr, DBConfig *config = nullptr);
// 	DUCKDB_API explicit DuckDB(const string &path, DBConfig *config = nullptr);
	~DuckDB();

// 	//! Reference to the actual database instance
// 	// shared_ptr<DatabaseInstance> instance;

public:
	template <class T>
	void LoadExtension() {
		T extension;
		extension.Load(*this);
	}

// 	DUCKDB_API FileSystem &GetFileSystem();

	 idx_t NumberOfThreads();
	 static const char *SourceID();
	 static const char *LibraryVersion();
};

struct DatabaseData {
	DatabaseData() : database(nullptr) {
	}
	~DatabaseData() {
		if (database) {
			delete database;
		}
	}

	DuckDB *database;
};
