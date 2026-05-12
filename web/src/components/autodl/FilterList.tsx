import React from 'react';
import { Edit2, Trash2 } from 'lucide-react';
import { AutodlFilter } from '../../hooks/useAutodl';
import { FilterForm } from './FilterForm';

interface FilterListProps {
    filters: AutodlFilter[];
    editingIndex: number | null;
    editForm: AutodlFilter | null;
    setEditingIndex: (index: number | null) => void;
    setEditForm: (form: AutodlFilter | null) => void;
    handleEdit: (index: number) => void;
    handleDelete: (index: number) => void;
    handleSaveEdit: () => void;
}

export const FilterList: React.FC<FilterListProps> = ({
    filters,
    editingIndex,
    editForm,
    setEditingIndex,
    setEditForm,
    handleEdit,
    handleDelete,
    handleSaveEdit
}) => {
    return (
        <div className="flex flex-col gap-4">
            {filters.map((filter, index) => (
                editingIndex === index ? (
                    <FilterForm 
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
            {editingIndex === filters.length && editForm && (
                <FilterForm 
                    form={editForm} 
                    setForm={setEditForm} 
                    onSave={handleSaveEdit} 
                    onCancel={() => { setEditingIndex(null); setEditForm(null); }} 
                />
            )}
        </div>
    );
};
