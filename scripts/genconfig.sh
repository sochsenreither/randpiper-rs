set -e

cargo build --package=genconfig --release
mkdir -p ./test/config
./target/release/genconfig -o binary -n 3 -d 100 -b 1 -C 6000 -P 7000 -t ./test/3/
