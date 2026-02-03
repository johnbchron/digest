# digest

This program takes a list of RSS URLs in `feeds.txt`, and fetches and parses
them on startup. It serves a web server on port `3000`, and the root route
serves a page with a list of recent posts (right now the last 5 days). There is
currently no mechanism to re-fetch feeds as time goes on.
