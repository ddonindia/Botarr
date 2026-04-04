import { useState, useCallback } from 'react';

type ToastType = 'success' | 'error';

interface ToastState {
    message: string;
    type: ToastType;
    visible: boolean;
}

export function useToast() {
    const [toast, setToast] = useState<ToastState>({ message: '', type: 'success', visible: false });

    const showToast = useCallback((message: string, type: ToastType) => {
        setToast({ message, type, visible: true });
    }, []);

    const hideToast = useCallback(() => {
        setToast(prev => ({ ...prev, visible: false }));
    }, []);

    return { toast, showToast, hideToast };
}
