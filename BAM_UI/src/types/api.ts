export type ApiResponse<T> = {
    success: boolean;
    data?: T | null;
    message?: string | null;
    error?: string | null;
}

export type ListResponse<T> = ApiResponse<T[]>;