#find src/ | entr -c bash -c "cargo run -- -p -k test.keys -k test2.keys test.properties result.properties"
find src/ | entr -c bash -c "cargo run -- test.properties result.properties"
#find src/ | entr -c bash -c "cargo test"
