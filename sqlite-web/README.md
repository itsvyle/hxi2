# sqlite-web

This is a go wrapper around [sqlite-web](https://github.com/coleifer/sqlite-web) to allow using multiple files, all protected behind the hxi2 authentication.

Only one user can use the web interface at a time though, as it will only run a single sqlite-web process at a given time. The running instance will also automatically shut down after 10 minutes of inactivity
