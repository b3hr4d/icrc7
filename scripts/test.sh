cp .dfx/local/canisters/factory/service.did.js .dfx/local/canisters/factory/factory.did.test.cjs
sed -i '' 's/export//g' .dfx/local/canisters/factory/factory.did.test.cjs
echo "module.exports = { idlFactory };" >> .dfx/local/canisters/factory/factory.did.test.cjs

cp ./src/declarations/icrc7/icrc7.did.js ./tests/icrc7.did.test.cjs
sed -i '' 's/export//g' ./tests/icrc7.did.test.cjs
echo "module.exports = { idlFactory };" >> ./tests/icrc7.did.test.cjs

tape tests/icrc7.test.cjs