-- plugins/autodl.lua
-- A comprehensive autodl plugin for Botarr

local config = nil

local function load_config()
    local success, result = pcall(botarr.get_autodl_filters)
    if success and type(result) == "table" then
        config = result
    else
        config = { filters = {} }
    end
end

-- Load configuration on startup
load_config()

local function setup_monitors()
    if config and config.enabled == true and config.filters then
        for _, filter in ipairs(config.filters) do
            if filter.network and filter.channels then
                for _, channel in ipairs(filter.channels) do
                    botarr.monitor_channel("autodl.lua", filter.network, channel)
                end
            end
        end
    end
end

-- Register monitored channels on startup
setup_monitors()

-- Listen for config changes from the API to dynamically reload and register monitors
botarr.signal_add("config_changed", function(plugin_name)
    if plugin_name == "autodl.lua" then
        botarr.print("autodl.lua", "Reloading config and restarting monitors...")
        load_config()
        setup_monitors()
    end
end)

-- Helper function to check if a value is in a list, or if the list contains a matching pattern
local function matches_list(value, list)
    if not list or #list == 0 then return true end
    for _, item in ipairs(list) do
        if botarr.regex_match(item, value) then
            return true
        end
    end
    return false
end

local function on_irc_message(network, channel, nick, message)
    -- Reload config live so UI changes apply immediately
    load_config()

    if not config or config.enabled ~= true or not config.filters then return end

    -- Global filter: Ignore SceneP2P bots with |P| in their name
    if string.find(string.lower(network), "scenep2p", 1, true) and string.find(string.lower(nick), "|p|", 1, true) then
        return
    end

    for _, filter in ipairs(config.filters) do
        -- 1. Check network
        if filter.network and network ~= filter.network then goto continue end
        
        -- 2. Check channel
        if not matches_list(channel, filter.channels) then goto continue end
        
        -- 3. Check bot (nick)
        if not matches_list(nick, filter.bots) then goto continue end

        -- 4. Check keywords (Smart Match - Order independent & case insensitive)
        if filter.keywords and type(filter.keywords) == "table" then
            local lower_msg = string.lower(message)
            local all_matched = true
            for _, word in ipairs(filter.keywords) do
                -- The `1, true` arguments make string.find use plain text search, not patterns
                if not string.find(lower_msg, string.lower(word), 1, true) then
                    all_matched = false
                    break
                end
            end
            if not all_matched then goto continue end
        end

        -- 5. Check match pattern (Advanced Regex Pattern)
        if filter.match and not botarr.regex_match(filter.match, message) then goto continue end

        -- 6. Check exclude pattern
        if filter.exclude and botarr.regex_match(filter.exclude, message) then goto continue end

        -- If all checks pass, we have a match!
        botarr.print("autodl.lua", "Autodl match for filter '" .. (filter.name or "Unnamed") .. "'!")
        
        -- Extract pack number
        local pack_num = string.match(message, "Pack #(%d+)") or string.match(message, "#(%d+)")
        if pack_num then
            local url = "irc://" .. network .. "/" .. channel .. "/" .. nick .. "/" .. pack_num
            botarr.print("autodl.lua", "Queuing: " .. url .. " | Msg: " .. message)
            
            -- Heuristic: The actual release name is usually the longest contiguous word in the announce string
            local guessed_filename = nil
            local max_len = 0
            for word in string.gmatch(message, "%S+") do
                if string.len(word) > max_len then
                    max_len = string.len(word)
                    guessed_filename = word
                end
            end
            
            -- Strip some common wrapping brackets if they stuck to the filename
            if guessed_filename then
                guessed_filename = string.gsub(guessed_filename, "^%[", "")
                guessed_filename = string.gsub(guessed_filename, "%]$", "")
            end

            botarr.queue(url, guessed_filename)
        else
            botarr.print("autodl.lua", "Could not extract pack number from message: " .. message)
        end

        ::continue::
    end
end

-- Add command to reload config via IRC or future UI (optional, for now we just load once)
-- But we can hook into events.
botarr.signal_add("irc_message", on_irc_message)
