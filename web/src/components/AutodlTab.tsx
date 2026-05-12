import React, { useState } from 'react';
import { Plus } from 'lucide-react';
import { useAutodl, AutodlFilter } from '../hooks/useAutodl';
import { FilterList } from './autodl/FilterList';

export const AutodlTab: React.FC = () => {
    const { filters, isEnabled, isLoading, saveFilters, toggleEnabled } = useAutodl();
    const [editingIndex, setEditingIndex] = useState<number | null>(null);
    const [editForm, setEditForm] = useState<AutodlFilter | null>(null);

    const handleAdd = () => {
        setEditingIndex(filters.length);
        setEditForm({
            name: "New Filter",
            network: "irc.scenep2p.net",
            channels: ["#THE.SOURCE"],
            bots: [".*"],
            match: "",
            exclude: "",
            keywords: []
        });
    };

    const handleEdit = (index: number) => {
        setEditingIndex(index);
        setEditForm({ ...filters[index] });
    };

    const handleDelete = (index: number) => {
        if (confirm("Are you sure you want to delete this filter?")) {
            const newFilters = filters.filter((_, i) => i !== index);
            saveFilters(newFilters);
        }
    };

    const handleSaveEdit = async () => {
        if (!editForm) return;
        const newFilters = [...filters];
        if (editingIndex! >= newFilters.length) {
            newFilters.push(editForm);
        } else {
            newFilters[editingIndex!] = editForm;
        }
        
        const success = await saveFilters(newFilters);
        if (success) {
            setEditingIndex(null);
            setEditForm(null);
        }
    };

    return (
        <div className="flex flex-col gap-6 h-full p-4 max-w-5xl mx-auto w-full animate-fade-in">
            <div className="flex justify-between items-center bg-black/20 p-6 rounded-xl border border-white/5 shadow-2xl backdrop-blur-xl">
                <div>
                    <div className="flex items-center gap-4 mb-1">
                        <h2 className="text-2xl font-semibold">Auto-Download Plugin</h2>
                        <label className="relative inline-flex items-center cursor-pointer">
                            <input type="checkbox" className="sr-only peer" checked={isEnabled} onChange={toggleEnabled} />
                            <div className="w-11 h-6 bg-white/10 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>
                            <span className="ml-3 text-sm font-medium text-secondary">{isEnabled ? 'Enabled' : 'Disabled'}</span>
                        </label>
                    </div>
                    <p className="text-sm text-secondary">Manage rules to automatically queue XDCC transfers based on IRC announcements.</p>
                </div>
                <button
                    onClick={handleAdd}
                    className="flex items-center gap-2 px-4 py-2 bg-primary/20 text-primary hover:bg-primary/30 rounded-lg transition-colors border border-primary/30"
                >
                    <Plus size={16} />
                    Add Filter
                </button>
            </div>

            {isLoading ? (
                <div className="flex justify-center items-center h-48">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                </div>
            ) : filters.length === 0 && editingIndex === null ? (
                <div className="text-center py-12 bg-black/20 rounded-xl border border-white/5">
                    <p className="text-secondary mb-4">No filters configured.</p>
                    <button onClick={handleAdd} className="text-primary hover:underline">Create your first filter</button>
                </div>
            ) : (
                <FilterList 
                    filters={filters}
                    editingIndex={editingIndex}
                    editForm={editForm}
                    setEditingIndex={setEditingIndex}
                    setEditForm={setEditForm}
                    handleEdit={handleEdit}
                    handleDelete={handleDelete}
                    handleSaveEdit={handleSaveEdit}
                />
            )}
        </div>
    );
};
