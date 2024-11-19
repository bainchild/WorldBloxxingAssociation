roblox api replacement using rust  
(axum, http, tower, & surrealdb)
---
To run, just launch a surrealdb instance on port 8000,  
redirect all DNS queries for roblox subdomains to localhost  
and then launch the program.  

If the program fails to launch, and you're running linux, you may
need to give the executable `CAP_NET_BIND_SERVICE` capability to bind
to port 80 and port 443:  
`sudo setcap +CAP_NET_BIND_SERVICE path/to/bloxxing_match`  
  
A tool is also provided to generate a `cacert.pem` file for
roblox installations from a public key present in the `cert/` directory.
