import { useState, useEffect, useCallback } from 'react';
import { useToast } from './useToast';

export interface AutodlFilter {
    name: string;
    network: string;
    channels: string[];
    bots: string[];
    match: string;
    exclude: string;
    keywords: string[];
}

export const useAutodl = () => {
    const [filters, setFilters] = useState<AutodlFilter[]>([]);
    const [isEnabled, setIsEnabled] = useState<boolean>(false);
    const [isLoading, setIsLoading] = useState(true);
    const { showToast } = useToast();

    const fetchFilters = useCallback(async () => {
        setIsLoading(true);
        try {
            const res = await fetch('/api/plugins/autodl/filters');
            const data = await res.json();
            setFilters(data.filters || []);
            setIsEnabled(data.enabled ?? false);
        } catch (e) {
            showToast("Failed to load filters", "error");
        } finally {
            setIsLoading(false);
        }
    }, [showToast]);

    useEffect(() => {
        fetchFilters();
    }, [fetchFilters]);

    const saveFilters = async (newFilters: AutodlFilter[]) => {
        try {
            const res = await fetch('/api/plugins/autodl/filters', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ enabled: isEnabled, filters: newFilters })
            });
            if (res.ok) {
                setFilters(newFilters);
                showToast("Filters saved successfully!", "success");
                return true;
            } else {
                showToast("Failed to save filters", "error");
                return false;
            }
        } catch (e) {
            showToast("Failed to save filters", "error");
            return false;
        }
    };

    const toggleEnabled = async () => {
        const newEnabled = !isEnabled;
        setIsEnabled(newEnabled);
        try {
            const res = await fetch('/api/plugins/autodl/filters', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ enabled: newEnabled, filters: filters })
            });
            if (res.ok) {
                showToast(newEnabled ? "Autodl plugin enabled" : "Autodl plugin disabled", "success");
            } else {
                showToast("Failed to toggle plugin", "error");
                setIsEnabled(!newEnabled); // Revert
            }
        } catch (e) {
            showToast("Failed to toggle plugin", "error");
            setIsEnabled(!newEnabled); // Revert
        }
    };

    return {
        filters,
        isEnabled,
        isLoading,
        saveFilters,
        toggleEnabled
    };
};
