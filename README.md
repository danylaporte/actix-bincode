# Overview
This crate provides an extractor for working with Bincode.
It closely mirrors the API for JSON extraction within Actix-Web, and in fact borrows most of it's
code from Actix-Web.

# Example
```rust
    use actix_bincode::Bincode;

    #[derive(serde::Deserialize)]
    struct User {
        name: String,
    }

    #[derive(serde::Serialize)]
    struct Greeting {
        inner: String,
    }

    #[actix_web::get("/users/hello")]
    pub async fn greet_user(user: Bincode<User>) -> Bincode<Greeting> {
        let name: &str = &user.name;
        let inner: String = format!("Hello {}!", name);
        Bincode(Greeting { inner })
    }
```

# Contributing
If you have a bug report or feature request, create a new GitHub issue.

Pull requests are welcome.