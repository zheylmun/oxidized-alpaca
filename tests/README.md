# Oxidized Alpaca Integration Test Suite

This integration test suite is designed to test the functionality of the oxidized-alpaca library against the Alpaca API.
It utilizes the paper api key and secret provided set in the local environment, and makes a series of real calls.
Please be aware that this test suite makes real calls to the Alpaca API, and may result in paper trades on the calling account.
The test suite will not run without the proper environment variables set.
The calls are organized in such a way as to try to clean up after themselves, but please be aware that any failures might
result in orphaned orders or positions on the account.
