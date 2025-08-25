build-devenv:
    docker build -t myhomelab-devenv -f .docker/devenv.dockerfile .

run-devenv: build-devenv
    docker run --rm -it \
        -p 2222:2222 \
        -v ~/.ssh/id_rsa.pub:/root/.ssh/authorized_keys \
        -v $(pwd)/.git:/code/.git:ro \
        -v $(pwd)/src:/code/src \
        -v $(pwd)/systemd:/code/systemd \
        -v $(pwd)/.gitignore:/code/.gitignore \
        -v $(pwd)/Cargo.toml:/code/Cargo.toml \
        -v $(pwd)/target/devenv:/code/target \
        myhomelab-devenv
