# Environments

This was code initially developed for my [university thesis](https://github.com/vagos/asp-games). The idea was to
create a system where a developer could give a high level description of a
scene and then a scene would be generated and displayed based on some
optimization algorithm. The final thesis ended up generalising the problem of
"high level description" to game mechanics and game content generators. This
application was also scrapped in favor of embedding an Answer Set Programming
solver to a ready-made game engine.

I am releasing this code base more of as a reminder to continue working on this
in the future.

## Building/Running 

Run `cargo run` inside the `src` directory and it should work.
Edit the `src/main.lua` file to specify the scene that is generated.
