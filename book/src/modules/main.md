
# Modules

Sometimes you want to split your code to multiple files - modules are just the thing you need!

## Exporting

You can export the following things:

- struct declarations
- enum declarations
- static variables
- type aliases

```
export enum SongGenre {
    Rock,
    Pop,
    Metal,
    Jazz,
    Rap,
    HipHop
}

export struct Song {
    const lyrics: str,
    const genre: SongGenre
}

export static create_song = fn(lyrics: str, genre: SongGenre) -> Song {
    new Song { lyrics, genre } 
}
```

## Importing

```
import "./path/to/songs"

main {
    let bloodmoney = create_song("what do you believe in?", SongGenre::Metal);
}
```

### `as` binding

```
import "./path/to/songs" as Songs;

main {
    let bloodmoney = Songs::create_song("what do you believe in?", Songs::SongGenre::Metal);
}
```