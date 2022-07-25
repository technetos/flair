# Flair - A tiny url service

# Usage

Run the `flair_data` RPC service

```sh
cd flair_data
cargo r --release -- -server_addr=127.0.0.1:3001
```

Run the `flair_server` REST service

```sh
cd flair_server
cargo r --release -- -search_service_addr=http://127.0.0.1:3001 -server_addr=127.0.0.1:3000
```

Open your browser and navigate to http://127.0.0.1:3000/1/search/test
