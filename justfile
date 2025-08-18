build-devenv:
    docker build -t myhomelab-devenv -f .docker/devenv.dockerfile .

run-devenv: build-devenv
    docker run --rm -it \
        -p 2222:2222 \
        -v ~/.ssh/id_rsa.pub:/root/.ssh/authorized_keys \
        -v $(pwd)/.git:/code/.git:ro \
        -v $(pwd)/adapter:/code/adapter \
        -v $(pwd)/client:/code/client \
        -v $(pwd)/domain:/code/domain \
        -v $(pwd)/prelude:/code/prelude \
        -v $(pwd)/sensor:/code/sensor \
        -v $(pwd)/server:/code/server \
        -v $(pwd)/.editorconfig:/code/.editorconfig \
        -v $(pwd)/.gitignore:/code/.gitignore \
        -v $(pwd)/Cargo.toml:/code/Cargo.toml \
        -v $(pwd)/rustfmt.toml:/code/rustfmt.toml \
        -v $(pwd)/target/devenv:/code/target \
        myhomelab-devenv
