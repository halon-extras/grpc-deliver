# Build instructions

```
export HALON_REPO_USER=exampleuser
export HALON_REPO_PASS=examplepass
docker compose -p grpc-deliver up --build
docker compose -p grpc-deliver down --rmi local
```
