#!/bin/bash
export JWT_KEY_PATH=./security/jwt_private_key.der
export SURREAL_HOSTNAME=test-instance-06aaqih481vdd93svatdjo1r00.aws-euw1.surreal.cloud
export SURREAL_NAMESPACE=Hackerwear
export SURREAL_USERNAME=test_admin
export SURREAL_PASSWORD=admin
export SURREAL_DATABASE=Products
cargo run
