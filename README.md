# Jikan API Discord Bot in Rust

This Discord bot is written in Rust using the [serenity](https://docs.rs/serenity/latest/serenity/) framework. It fetches and displays anime information from [MyAnimeList](https://myanimelist.net) via the public RESTful [Jikan API](https://jikan.moe).

> 🧠 _Note: This bot features subtle aesthetic and functional references to_ **Serial Experiments Lain**. Try `!lain` or `!lain recommendations` commands to experience the Wired.

---

## 📦 Features

- Search for anime by title
- Retrieve detailed anime info by MyAnimeList ID
- Get anime recommendations based on a given MAL ID
- Tracks user search queries locally in `user_queries.json`
- Special `!lain` commands dedicated to _Serial Experiments Lain_ fans

---

## 🔧 Commands

### `!anime <anime name>`

Searches MyAnimeList for anime titles matching the query.

**Example usage:**

```
!anime cyberpunk
```

> 📝 User queries are logged per user in a local JSON file named `data/user_queries.json`.

---

### `!anime details <MAL_ID>`

Fetches detailed metadata about a specific anime via its MyAnimeList ID.

**Returned data includes:**

- Titles (original, English, Japanese, synonyms)
- Synopsis (truncated at 1000 characters)
- Poster image
- Type, Source, Episode count
- Score, Popularity, Rank
- Airing status and dates
- Duration, Rating
- Favorites, Members count
- Trailer link (if available)

**Example usage:**

```
!anime details 339
```

> This fetches full info for _Serial Experiments Lain_.

---

### `!anime recommendations <MAL_ID>`

Returns up to 5 recommended anime based on the specified MAL ID.

**Example usage:**

```
!anime recommendations 339
```

> Recommendations related to _Serial Experiments Lain_ will be displayed.

---

### `!lain` and `!lain recommendations`

Built-in shortcuts for _Serial Experiments Lain_:

- `!lain`: Displays detailed info for MAL ID 339
- `!lain recommendations`: Shows 5 recommended anime related to Lain

---

## 🎨 How It Looks

### 🆘 `!help` Command Output

Here’s how the `!help` command appears inside Discord:

![Help Command Screenshot](git-docs/images/anime_bot_help.png)

---

### 🎬 `!anime details 10620` Example Output

An example output for Mirai Nikki anime details:

![Anime Details Part 1](git-docs/images/anime_details_1.png)  
![Anime Details Part 2](git-docs/images/anime_details_2.png)

---

### 🎬 Anime Recommendations Example

How recommendations are presented:

![Anime Recommendations](git-docs/images/anime_recommendations.png)

---

## 🧠 Internals

### File: `user_queries.json`

This file stores past search queries by user, structured as:

```
{
"user_id_1": ["search1", "search2"],
"user_id_2": ["another search"]
}
```

It’s automatically created if missing.

---

### API Integration: [Jikan API](https://jikan.moe)

All data is fetched asynchronously via Jikan API endpoints such as:

- `GET /anime/{id}` → anime details
- `GET /anime/{id}/recommendations` → recommendations
- `GET /anime?q=search_term` → search results

Jikan enables access to MAL data **without OAuth**.

---

## 🚀 Bot Behavior

- Listens and responds to all non-bot messages starting with `!anime`, `!lain`, or `!help`
- Validates user inputs and handles errors gracefully
- Uses rich embed messages to present information neatly
- Logs connection status via `ready()` and `presence_update()` events

---

## 🔮 The Lain Aesthetic

This bot contains subtle references to _Serial Experiments Lain_ throughout:

- Special `!lain` commands
- Help text referencing “The Wired”
- Quotes and thematic touches in logs and responses

> _“We are all connected.”_

---

## 🛠️ Development Notes

### Dependencies

- `serenity` — Discord bot framework
- `tokio` — async runtime
- `serde` + `serde_json` — serialization
- `reqwest` (in `jikan_api` module) — HTTP client

### Running the Bot

1. Create `.env` file with your Discord bot token:

```
DISCORD_TOKEN="your_token_here"
```

2. Build the project:

```
cargo build
```

3. Run it:

```
cargo run
```

Your bot will now be active and responding in your Discord server.

---

## 🧩 Extending and Customizing

You can add support for more Jikan API endpoints like genres, characters, or manga by expanding the bot’s functionality. Explore the API docs at [https://jikan.moe](https://jikan.moe).

---

## 🌀 Welcome to The Wired

This bot is more than a tool — it’s a homage to _Serial Experiments Lain_ and the interconnected nature of data, identity, and being online.

> **“No matter where you are… everyone is always connected.”**

---
