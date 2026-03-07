import { PUBLIC_API_URL } from '$env/static/public';

const BASE = PUBLIC_API_URL;

export interface UploadInitResponse {
    video_id: string;
    upload_url: string;
}

export interface UploadCompleteResponse {
    video_id: string;
    watch_url: string;
}

export interface VideoResponse {
    id: string;
    filename: string;
    mime_type: string | null;
    status: string;
    created_at: string;
}

export interface StreamUrlResponse {
    stream_url: string;
    mime_type: string | null;
    filename: string;
}

// Step 1: Tell the backend we want to uplaod - get a presigned PUT URL
export async function initUpload(
    filename: string,
    size_bytes: number,
    mime_type: string
): Promise<UploadInitResponse> {
    const res = await fetch(`${BASE}/api/upload/init`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ filename, size_bytes, mime_type })
    });

    if (!res.ok) {
        const err = await res.json().catch(() => ({ error: 'Unknown error ' }));
        throw new Error(err.error ?? 'Upload init failed');
    }

    return res.json();
}

// Step 2: PUT the file directly to R2 using the presigned URL
export async function uploadToR2(
    uploadUrl: string,
    file: File,
    onProgress: (pct: number) => void
): Promise<void> {
    await new Promise<void>((resolve, reject) => {
        const xhr = new XMLHttpRequest();
        xhr.open('PUT', uploadUrl);
        // Don't set Content-Type - let the presigned URL handle it
        xhr.upload.onprogress = (e) => {
            if (e.lengthComputable) onProgress(Math.round((e.loaded / e.total) * 100));
        };
        xhr.onload = () => (xhr.status === 200 ? resolve() : reject(new Error(`R2 upload failed: ${xhr.status}`)));
        xhr.onerror = () => reject(new Error('Network error during upload'));
        xhr.send(file);
    });
}

// Step 3: Tell the backend upload is done -> marks video as ready
export async function completeUpload(videoId: string): Promise<UploadCompleteResponse> {
    const res = await fetch(`${BASE}/api/upload/complete/${videoId}`, { method: 'POST' });
    if (!res.ok) throw new Error('Failed to complete upload');
    return res.json();
}

// Get video metadata
export async function getVideo(videoId: string): Promise<VideoResponse> {
    const res = await fetch(`${BASE}/api/video/${videoId}`);
    if (!res.ok) throw new Error('Video not found');
    return res.json();
}

// Get a fresh presigned stream URL
export async function getStreamUrl(videoId: string): Promise<StreamUrlResponse> {
    const res = await fetch(`${BASE}/api/video/${videoId}/stream`);
    if (!res.ok) throw new Error('Video not available for streaming');
    return res.json();
}