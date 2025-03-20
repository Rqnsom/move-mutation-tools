#!/bin/bash

cd target/release;

ZIP_FILE=mutation-tools.zip
zip $ZIP_FILE move-mutation-test move-mutator move-spec-test
echo "Zipping release: $ZIP_FILE"

cp $ZIP_FILE ../..
