import React, { useState, useEffect } from 'react';
import { Plus, Trash2, Edit2, Save } from 'lucide-react';
import { useToast } from '../hooks/useToast';

interface AutodlFilter {
    name: string;
    network: string;
    channels: string[];
    bots: string[];
    match: string;
    exclude: string;
    keywords: string[];
}

export const AutodlTab: React.FC = () => {
    const [filters, setFilters] = useState<AutodlFilter[]>([]);
    const [isEnabled, setIsEnabled] = useState<boolean>(false);
    const [isLoading, setIsLoading] = useState(true);
    const [editingIndex, setEditingIndex] = useState<number | null>(null);
    const [editForm, setEditForm] = useState<AutodlFilter | null>(null);
    const { showToast } = useToast();

    useEffect(() => {
        fetchFilters();
    }, []);

    const fetchFilters = async () => {
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
    };

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
            } else {
                showToast("Failed to save filters", "error");
            }
        } catch (e) {
            showToast("Failed to save filters", "error");
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

    const handleSaveEdit = () => {
        if (!editForm) return;
        const newFilters = [...filters];
        if (editingIndex! >= newFilters.length) {
            newFilters.push(editForm);
        } else {
            newFilters[editingIndex!] = editForm;
        }
        saveFilters(newFilters);
        setEditingIndex(null);
        setEditForm(null);
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
                <div className="flex flex-col gap-4">
                    {filters.map((filter, index) => (
                        editingIndex === index ? (
                            <FilterEditor 
                                key={`edit-${index}`}
                                form={editForm!} 
                                setForm={setEditForm} 
                                onSave={handleSaveEdit} 
                                onCancel={() => { setEditingIndex(null); setEditForm(null); }} 
                            />
                        ) : (
                            <div key={index} className="bg-black/20 rounded-xl border border-white/5 p-5 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 hover:border-white/10 transition-colors">
                                <div className="flex-1">
                                    <h3 className="text-lg font-medium text-white/90 flex items-center gap-3">
                                        {filter.name}
                                        <span className="text-xs px-2 py-0.5 rounded-full bg-blue-500/20 text-blue-400 border border-blue-500/30">
                                            {filter.network}
                                        </span>
                                    </h3>
                                    <div className="mt-2 grid grid-cols-2 gap-x-8 gap-y-2 text-sm text-secondary">
                                        <div><span className="opacity-50">Channels:</span> {filter.channels.join(', ')}</div>
                                        <div><span className="opacity-50">Bots:</span> {filter.bots.join(', ')}</div>
                                        {filter.match && <div><span className="opacity-50">Match:</span> <code className="bg-black/40 px-1.5 py-0.5 rounded">{filter.match}</code></div>}
                                        {filter.keywords && filter.keywords.length > 0 && <div><span className="opacity-50">Keywords:</span> <code className="bg-black/40 px-1.5 py-0.5 rounded">{filter.keywords.join(', ')}</code></div>}
                                    </div>
                                </div>
                                <div className="flex gap-2">
                                    <button onClick={() => handleEdit(index)} className="p-2 text-secondary hover:text-white hover:bg-white/10 rounded transition-colors" title="Edit">
                                        <Edit2 size={16} />
                                    </button>
                                    <button onClick={() => handleDelete(index)} className="p-2 text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded transition-colors" title="Delete">
                                        <Trash2 size={16} />
                                    </button>
                                </div>
                            </div>
                        )
                    ))}
                    {editingIndex === filters.length && (
                        <FilterEditor 
                            form={editForm!} 
                            setForm={setEditForm} 
                            onSave={handleSaveEdit} 
                            onCancel={() => { setEditingIndex(null); setEditForm(null); }} 
                        />
                    )}
                </div>
            )}
        </div>
    );
};

const FilterEditor: React.FC<{
    form: AutodlFilter,
    setForm: (val: AutodlFilter) => void,
    onSave: () => void,
    onCancel: () => void
}> = ({ form, setForm, onSave, onCancel }) => {
    return (
        <div className="bg-[#111115] rounded-xl border border-primary/30 p-5 shadow-[0_0_15px_rgba(var(--primary-rgb),0.1)]">
            <div className="flex justify-between items-center mb-4 border-b border-white/10 pb-3">
                <h3 className="font-semibold text-lg text-primary">Edit Filter</h3>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
                <div>
                    <label className="block text-xs uppercase tracking-wider text-secondary mb-1">Filter Name</label>
                    <input type="text" value={form.name} onChange={e => setForm({...form, name: e.target.value})} className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm focus:border-primary/50 outline-none" />
                </div>
                <div>
                    <label className="block text-xs uppercase tracking-wider text-secondary mb-1">Network</label>
                    <input type="text" value={form.network} onChange={e => setForm({...form, network: e.target.value})} className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm focus:border-primary/50 outline-none" />
                </div>
                <div>
                    <label className="block text-xs uppercase tracking-wider text-secondary mb-1">Channels (comma-separated)</label>
                    <input type="text" value={form.channels.join(', ')} onChange={e => setForm({...form, channels: e.target.value.split(',').map(s=>s.trim()).filter(Boolean)})} className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm focus:border-primary/50 outline-none" placeholder="#channel1, #channel2" />
                </div>
                <div>
                    <label className="block text-xs uppercase tracking-wider text-secondary mb-1">Bots (comma-separated regex)</label>
                    <input type="text" value={form.bots.join(', ')} onChange={e => setForm({...form, bots: e.target.value.split(',').map(s=>s.trim()).filter(Boolean)})} className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm focus:border-primary/50 outline-none" placeholder=".*" />
                </div>
                <div>
                    <label className="block text-xs uppercase tracking-wider text-secondary mb-1">Lua Match Pattern</label>
                    <input type="text" value={form.match || ''} onChange={e => setForm({...form, match: e.target.value})} className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm focus:border-primary/50 outline-none" placeholder="SomeMovie.*1080p" />
                </div>
                <div>
                    <label className="block text-xs uppercase tracking-wider text-secondary mb-1">Lua Exclude Pattern</label>
                    <input type="text" value={form.exclude || ''} onChange={e => setForm({...form, exclude: e.target.value})} className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm focus:border-primary/50 outline-none" placeholder="FRENCH|GERMAN" />
                </div>
                <div className="md:col-span-2">
                    <label className="block text-xs uppercase tracking-wider text-secondary mb-1">Smart Keywords (comma-separated)</label>
                    <input type="text" value={(form.keywords || []).join(', ')} onChange={e => setForm({...form, keywords: e.target.value.split(',').map(s=>s.trim()).filter(Boolean)})} className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm focus:border-primary/50 outline-none" placeholder="event, 2026, world, cup" />
                    <p className="text-xs text-secondary/60 mt-1">Smart keywords are case-insensitive and can appear in any order in the announcement.</p>
                </div>
            </div>

            <div className="flex justify-end gap-3 pt-4 border-t border-white/10">
                <button onClick={onCancel} className="px-4 py-2 text-sm text-secondary hover:text-white transition-colors">Cancel</button>
                <button onClick={onSave} className="flex items-center gap-2 px-6 py-2 bg-primary text-black font-medium rounded-lg hover:bg-primary/90 transition-colors shadow-[0_0_15px_rgba(var(--primary-rgb),0.3)]">
                    <Save size={16} /> Save Filter
                </button>
            </div>
        </div>
    );
};
