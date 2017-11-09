# Font Finder

Have you RIIR lately? This project is a clean Rust implementation of
[TypeCatcher](https://github.com/andrewsomething/typecatcher/), which took
about two days. It is a GTK3 application for browsing through and installing
fonts from [Google's font archive](https://fonts.google.com/) from the comfort
of your Linux desktop. Compared to TypeCatcher, which is written in Python,
Font Finder also enables the ability to filter fonts by their categories,
has zero Python runtime dependencies, and has much better performance &
resource consumption.

## Installation Instructions

```
cargo install --git https://github.com/mmstick/fontfinder
```

> When a new suite of GTK crates are released that also contains the
> webkit2gtk crate, the above installation instructions will be changed to
> `cargo install fontfinder`.

## Screenshots

### Default

![First Screenshot](screenshot01.png)

### Filtering Fonts w/ Search

![Second Screenshot](screenshot02.png)

### Filtering Fonts w/ Category

![Third Screenshot](screenshot03.png)

