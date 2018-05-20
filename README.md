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
make && sudo make install
```

## Requirements

- cargo (Rust 1.24.0)
- libwebkit2gtk-4.0-dev
- libgtk-3-dev

## Screenshots

### Filtering w/ Search

![First Screenshot](screenshot01.png)

### Filtering w/ Category

![Second Screenshot](screenshot02.png)

### Multiple Paragraphs

![Third Screenshot](screenshot03.png)

### Dark Preview

![Fourth Screenshot](screenshot04.png)
