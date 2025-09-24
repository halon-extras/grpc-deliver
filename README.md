# gRPC delivery plugin

## Testing

### Build and install the binaries

```
mkdir -p ./build/vscode && cd ./build/vscode && cmake ../../ && make && make install && cd ../../
```

### Start the test server

```
./build/vscode/target/release/grpc_server
```

### Restart the Halon installation

```
supervisorctl restart smtpd
```

### Tail the logs

```
supervisorctl tail -f smtpd stderr
```

### Send a test email

```
swaks --from test@halon.io --to test@halon.io --server 127.0.0.1 --port 25
```

## Building RPM/DEB packages

See the [build](./build) folder.
