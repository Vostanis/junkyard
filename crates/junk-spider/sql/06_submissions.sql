CREATE TABLE IF NOT EXISTS stock.ref_sec_endpoints (
	symbol_pk INT,
	form_type VARCHAR,
	url VARCHAR
);

CREATE TABLE IF NOT EXISTS stock.fact_sec_documents (
	filing_date DATE,
	form_type VARCHAR,
	url VARCHAR,
	xml XML
);

CREATE TABLE IF NOT EXISTS stock.fact_insider_transactions (
	transaction_date DATE,
	ticker VARCHAR,
	title VARCHAR,
	name VARCHAR,
	address VARCHAR,
	relationship VARCHAR,
	individual_flag BOOL,
	transaction_code VARCHAR,
	value_amount VARCHAR,
	direct_ownership_flag BOOL,
	indirect_ownership_type VARCAR
);
