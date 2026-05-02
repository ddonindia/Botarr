-- plugins/autodl.conf.lua
-- Native Lua configuration for Botarr autodl plugin

return {
    filters = {
        {
            name = "Test Announce Filter",
            network = "rizon",
            channels = { "#testannounce" },
            bots = { "TestBot", "ReleaseBot" },
            match = "TEST%.RELEASE%.1080p", -- Lua pattern match
            exclude = "FRENCH"
        },
        {
            name = "Generic Subs",
            network = "abjects",
            channels = { "#movie-releases" },
            bots = { ".*" }, -- Match any bot
            match = "SomeMovie.*1080p",
            exclude = nil
        },
        {
            name = "SceneP2P Movies",
            network = "irc.scenep2p.net",
            channels = { "#THE.SOURCE" },
            bots = { ".*" },
            match = "[Ee][Vv][Ee][Nn][Tt].*2026.*[Ww][Oo][Rr][Ll][Dd].*[Cc][Uu][Pp]", -- Match EVENT 2026 World Cup
            exclude = nil
        }
    }
}
