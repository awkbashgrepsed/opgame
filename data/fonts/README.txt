OPGAME UI FONT

Put a TrueType font at:

    data/fonts/SegoeUI.ttf

The Python build script copies this directory to the packaged runtime:

    dist/assets/fonts/SegoeUI.ttf

The game loads the packaged font from assets/fonts/ first. If no shipped font is present, it falls back to a platform system font.
