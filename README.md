# rusty-card (Myriad)

Collectible trading card game created for The Poster Database (TPDb) Community.

## About
Rusty-card is a Discord bot written in Rust using the serenity-rs crate. This is designed to be a trading card game similar to TYKHE or Karuta, but with a focus on custom posters and characters. The bot uses a Google Firebase backend for storing the images and for hosting the database. 

This bot is currently only in use on TPDb's discord server which can be joined [here](https://discord.com/invite/NARZqQX).

## Gameplay
### Rolling Cards
Myriad has two types of cards. First are *poster* cards. These are cards with poster artwork mostly created by members of the TPDb community. Second, are the *character* cards. These are cards with characters from movies, television and more. Here are the commands:

`!roll posters` to roll a poster card
`!roll characters` to roll a character card

You can also just use `!r c` etc. You can roll for a new card every 15 minutes.

### Trading Cards
Every card you roll will have an ID. This will usually consist of a 5-6 digit number. You can use this ID to give cards to other players. For example:

`!trade @user <card ID>` would transfer a card.

IDs are also listed in cards displayed in the inventory.

### Inventory
Once you have started a collection, you will probably want to see what cards you have. Your inventory is divided by card type and can be viewed with the commands:

`!inventory posters` or `!inventory characters`

From here you'll be able to find card IDs, quantity, and more. Inventory searching and filtering is planned, but not currently implemented.

##Contributing
If you would like to contribute to this project feel free to! The project is set up as a Devcontainer to run in Visual Studio Code so no manual installation of the rust toolchain is necessary to work on this project. To build this project build the Dockerfile in the root directory of this project.
