# mxh-auth

auth based around axum

### functionality:
- [x] login
- [x] logout
- [x] session based
- [x] persistent session
- [x] create a new user
- [ ] delete a user
- [ ] role based access
- [ ] topt
- [ ] config file

### usage:
1. create db
    - export DATABASE_URL="sqlite://./sessions.db"
    - sqlx database create
    - sqlx migrate add init
2. run
    - nixos : RUSTUP_TOOLCHAIN=stable cargo run
    - else: cargo run

### version:
- rustc 1.82.0
- cargo 1.82.0

### docs:
- sqlx: https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md