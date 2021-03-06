# TR Logic test task
![Rust](https://github.com/ilslv/trlogic_task/workflows/Rust/badge.svg) [![Azure deployment](https://github.com/ilslv/trlogic_task/actions/workflows/azure_cd.yml/badge.svg?branch=master)](https://bsu-lab4.azurewebsites.net) \
Simple webserver for uploading images written with [Actix Web]([https://actix.rs/](https://actix.rs/))
## Setup
To run locally
```
cargo run --release
```
To run inside docker container
```
docker-compose up
```
*Note: docker container size is `~34MB` and all images are saved inside `images`  volume*
## API
Description | Method |  URL | Body | Response
-|-|-|-|-
Home page | GET | localhost:8080 | none | index.html
Full images list | GET | localhost:8080/img/full | none | html page
Preview images list | GET | localhost:8080/img/preview | none | html page
Get full image | GET | localhost:8080/img/full/{imagename} | none | image
Get image preview | GET | localhost:8080/img/preview/{imagename} | none | image
Add images | POST | localhost:8080 | multipart/form-data | JSON with filenames
Add images | POST | localhost:8080 | JSON with urls or base64 images | JSON with filenames
## Additional requirements
- [x] Graceful shutdown
- [x] Dockerfile and docker-compose.yml
- [x]  Unit and integration tests, CI (Github Actions)
- [ ] FFI for image processing

