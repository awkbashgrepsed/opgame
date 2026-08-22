OPGAME UI FONT

Place a TrueType font here only if you have permission to redistribute it:

    data/fonts/<font-name>.ttf

The Python build script copies this directory to the packaged runtime:

    dist/assets/fonts/

The game loads a packaged font from assets/fonts/ first and otherwise falls back to a platform system font.

Do not commit proprietary fonts unless their license explicitly permits redistribution with this project.
