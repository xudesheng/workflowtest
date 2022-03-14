# https://github.com/clux/muslrust/blob/main/etc/profile.d/cargo.sh
if command -v cargo 2>/dev/null; then
    export PATH=/root/.cargo/bin:$PATH
    export RUSTUP_HOME=/root/.rustup
fi