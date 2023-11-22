killall -9 node-bft
timeout 900 ./randpiper-rs/target/release/node-bft -c ./randpiper-rs/test/64/nodes-$1.dat -d 8000 -i ./randpiper-rs/ips_file > output.log
