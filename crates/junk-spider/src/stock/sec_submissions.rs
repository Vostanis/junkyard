// Read all files per CIK, ie:
//
// 1. read submissions/CIK0000000000000.json
//
// 2. if 'files' IS NOT EMPTY -> read that next file
//
// 3. insert all url endings, per ticker, with file types
//
//      symbol_pk | file_type | url
//      ==================================
//      1         |    4      | ".../..."
//
// 4. webscraping process begins
//      4a. depend on CLI input for files types
//      4b. fetch all per symbol_pk at urls <<<<<<< ONLY 10 REQUESTS PER SECOND (9 for good measure)
//
// 5. parse xml documents for ownership updates
