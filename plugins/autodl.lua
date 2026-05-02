-- plugins/autodl.lua
-- A comprehensive autodl plugin for Botarr

local config = nil

local function load_config()
    local success, result = pcall(dofile, "plugins/autodl.conf.lua")
    if success and type(result) == "table" then
        config = result
        botarr.print("autodl.lua", "Autodl configuration loaded with " .. #config.filters .. " filters.")
    else
        botarr.print("autodl.lua", "Failed to load autodl.conf.lua. Ensure it exists and has valid Lua syntax.")
        config = { filters = {} }
    end
end

-- Load configuration on startup
load_config()

-- Register monitored channels
if config and config.filters then
    for _, filter in ipairs(config.filters) do
        if filter.network and filter.channels then
            for _, channel in ipairs(filter.channels) do
                botarr.monitor_channel("autodl.lua", filter.network, channel)
            end
        end
    end
end

-- Helper function to check if a value is in a list, or if the list contains a matching pattern
local function matches_list(value, list)
    if not list or #list == 0 then return true end
    for _, item in ipairs(list) do
        if string.match(value, item) then
            return true
        end
    end
    return false
end

local function on_irc_message(network, channel, nick, message)
    if not config or not config.filters then return end

    for _, filter in ipairs(config.filters) do
        -- 1. Check network
        if filter.network and network ~= filter.network then goto continue end
        
        -- 2. Check channel
        if not matches_list(channel, filter.channels) then goto continue end
        
        -- 3. Check bot (nick)
        if not matches_list(nick, filter.bots) then goto continue end

        -- 4. Check match pattern
        if filter.match and not string.find(message, filter.match) then goto continue end

        -- 5. Check exclude pattern
        if filter.exclude and string.find(message, filter.exclude) then goto continue end

        -- If all checks pass, we have a match!
        botarr.print("autodl.lua", "Autodl match for filter '" .. (filter.name or "Unnamed") .. "'!")
        
        -- Extract pack number
        local pack_num = string.match(message, "Pack #(%d+)") or string.match(message, "#(%d+)")
        if pack_num then
            local url = "irc://" .. network .. "/" .. channel .. "/" .. nick .. "/" .. pack_num
            botarr.print("autodl.lua", "Queuing: " .. url)
            botarr.queue(url)
        else
            botarr.print("autodl.lua", "Could not extract pack number from message: " .. message)
        end

        ::continue::
    end
end

-- Add command to reload config via IRC or future UI (optional, for now we just load once)
-- But we can hook into events.
botarr.signal_add("irc_message", on_irc_message)
