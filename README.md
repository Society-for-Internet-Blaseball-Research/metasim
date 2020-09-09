the plan:

1. reverse engineer blaseball's simulation
2. model future games
3. ???
4. *???*

we're chatting in #metasim on [the SIBR discord](https://discord.gg/XTZRmcb)

## running

Past game data is included in `game-data/` for seasons 4 and 5.

You also need SIBR roster / player archival data, which you can get with:

```
aws --no-sign-request s3 sync s3://blaseball-archive-iliana/compressed-hourly/ team-data/
```

The first time you run it will build up a time-series database of teams and players, so it will take some time. After that the database is cached.
