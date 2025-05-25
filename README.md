# Hackerwear-backend

This is a complete rewrite of the backend for the [Hackerwear App](https://github.com/SUGAT-BAGDE/HackerWear).

_The primary goal of this project was to learn Rust by building a real-world application._

## 🚧 Setting Up the Dev Environment

Use the package manager `cargo` to install dependencies:

```bash
cargo build
```

## 🚀 Running the App

To run the project locally with required environment variables, use a script like the one below:

```bash
#!/bin/bash
# dev_run.sh


# Path to the private key for JWT signing (auto-generated if missing)
export JWT_KEY_PATH=./security/jwt_private_key.der

# SurrealDB connection settings
export SURREAL_HOSTNAME=<hostname of surreal db instance>
export SURREAL_NAMESPACE=<namespace>
export SURREAL_USERNAME=<username>
export SURREAL_PASSWORD=<password>
export SURREAL_DATABASE=<database>

cargo run
```
> [!WARNING]
> The app will exit with a panic if SurrealDB connection settings are missing, as there are no defaults.

> [!NOTE]
> `.env` file mechanism has not been implemented yet.

> [!CAUTION] 
> Environment variables (including secrets) must currently be set manually or via a local script. This straightforward approach is recommended for local development to get started quickly without extra setup.
>
> In production, secret and env variables management will be handled by the containerization solution.
> 
> For security, this script is excluded from version control via `.gitignore`. Please keep your own copies private.

## 🔐  About .der Key File
The app uses a .der file (binary-encoded private key) to sign JWTs.

> [!IMPORTANT]
> You don’t need to generate it manually — the app creates it automatically on first run if it doesn’t exist.

By default, the key is saved to:

```bash
# In development
./security/jwt_private_key.der

# In production
/etc/myapp/jwt_private_key.der
```
You can override this location by setting the `JWT_KEY_PATH` environment variable.

## 🤝  Contributing

Pull requests are welcome. 

If possible, please include unit tests with your contributions. (I’m still learning, so your help with testing is appreciated!)
For major changes, please open an issue first to discuss what you'd like to change.
Please make sure to update tests as appropriate.

> [!TIP]
> You’re welcome to point out anything that could be improved, suggest better or more standard approaches, or even share innovative solutions.

👉 Also check out and contribute to the [HackerWear frontend](https://github.com/SUGAT-BAGDE/HackerWear) built with Next.js.

## 📄 License

This project is licensed under [GPL-v3 License](https://github.com/SUGAT-BAGDE/Hackerwear-backend?tab=GPL-3.0-1-ov-file).