export interface XdccSearchResult {
    server: string;
    bot: string;
    pack_number: number;
    file_size: number;
    file_name: string;
    downloads: number;
    channel: string;
}

export interface SearchResponse {
    results: XdccSearchResult[];
    count: number;
}

export type TransferStatus = 'pending' | 'connecting' | 'joining' | 'requesting' | 'downloading' | 'completed' | 'failed' | 'cancelled';
export type TransferPriority = 'low' | 'normal' | 'high' | 'urgent';

export interface XdccTransfer {
    id: string;
    url: {
        server: string;
        channel: string;
        bot: string;
        pack: number;
    };
    status: TransferStatus;
    file_name: Option<string>;
    file_size: Option<number>;
    downloaded: number;
    speed: number;
    progress: number;
    error: Option<string>;
    created_at: number;
    updated_at: number;
    priority: TransferPriority;
    retry_count: number;
    max_retries: number;
    queue_position: Option<number>;
}

// Helper type for TS since Rust Option ends up as null | T or just T?
// In Serde, Option is null if None.
type Option<T> = T | null;

export interface HistoryItem {
    id: string;
    file_name?: string;
    size?: number;
    updated_at: string; // ISO date string from Rust DateTime<Utc>
    status: TransferStatus;
}

export interface QueueStatus {
    queue_size: number;
    status: string;
}

export interface BotStats {
    bot_name: string;
    network: string;
    total_downloads: number;
    successful_downloads: number;
    failed_downloads: number;
    total_bytes: number;
    average_speed: number;
    reliability_score: number;
}

export interface NetworkConfig {
    host: string;
    port: number;
    ssl: boolean;
    autojoin_channels: string[];
    join_delay_secs: number;
}

export interface AppConfig {
    use_ssl: boolean;
    connect_timeout: number;
    general_timeout: number;
    proxy_enabled: boolean;
    proxy_url: string;
    nickname: string;
    username: string;
    realname: string;
    max_retries: number;
    retry_delay: number;
    queue_limit: number;
    passive_dcc: boolean;
    dcc_port_min: number;
    dcc_port_max: number;
    resume_enabled: boolean;
    enabled_providers: string[];
    results_per_page: number;
    search_timeout: number;
    networks: Record<string, NetworkConfig>;
}
