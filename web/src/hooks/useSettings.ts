import { useState, useEffect } from 'react';
import { AppConfig, NetworkConfig } from '../types';
import { useToast } from './useToast';

export const useSettings = () => {
    const [settings, setSettings] = useState<AppConfig | null>(null);
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);
    const { showToast } = useToast();

    useEffect(() => {
        fetchSettings();
    }, []);

    const fetchSettings = async () => {
        try {
            const res = await fetch('/api/settings');
            if (res.ok) {
                const data = await res.json();
                setSettings(data);
            }
        } catch (e) {
            showToast('Failed to load settings', 'error');
        } finally {
            setLoading(false);
        }
    };

    const saveSettings = async () => {
        if (!settings) return;

        setSaving(true);
        try {
            const res = await fetch('/api/settings', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(settings)
            });
            if (res.ok) {
                showToast('Settings saved!', 'success');
            } else {
                showToast('Failed to save settings', 'error');
            }
        } catch (e) {
            showToast('Failed to save settings', 'error');
        } finally {
            setSaving(false);
        }
    };

    const updateSetting = <K extends keyof AppConfig>(key: K, value: AppConfig[K]) => {
        if (settings) {
            setSettings({ ...settings, [key]: value });
        }
    };

    const addNetwork = async (newNetworkName: string) => {
        if (!newNetworkName.trim() || !settings) return false;

        const network: NetworkConfig = {
            host: `irc.${newNetworkName.toLowerCase()}.net`,
            port: 6697,
            ssl: true,
            autojoin_channels: [],
            join_delay_secs: 6,
            nickserv_password: ''
        };

        try {
            await fetch(`/api/settings/networks/${encodeURIComponent(newNetworkName)}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(network)
            });

            setSettings({
                ...settings,
                networks: { ...settings.networks, [newNetworkName]: network }
            });
            showToast('Network added', 'success');
            return true;
        } catch (e) {
            showToast('Failed to add network', 'error');
            return false;
        }
    };

    const deleteNetwork = async (name: string) => {
        if (!settings) return;

        try {
            await fetch(`/api/settings/networks/${encodeURIComponent(name)}`, { method: 'DELETE' });
            const newNetworks = { ...settings.networks };
            delete newNetworks[name];
            setSettings({ ...settings, networks: newNetworks });
            showToast('Network deleted', 'success');
        } catch (e) {
            showToast('Failed to delete network', 'error');
        }
    };

    const updateNetwork = (name: string, field: keyof NetworkConfig, value: any) => {
        if (!settings) return;
        setSettings({
            ...settings,
            networks: {
                ...settings.networks,
                [name]: { ...settings.networks[name], [field]: value }
            }
        });
    };

    return {
        settings,
        loading,
        saving,
        fetchSettings,
        saveSettings,
        updateSetting,
        addNetwork,
        deleteNetwork,
        updateNetwork
    };
};
