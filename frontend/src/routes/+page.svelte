<script lang="ts">
    import { initUpload, uploadToR2, completeUpload } from "$lib/api";

    type UploadState = "idle" | "uploading" | "completing" | "done" | "error";

    let state: UploadState = "idle";
    let progress = 0;
    let watchUrl = "";
    let videoId = "";
    let errorMessage = "";
    let isDragging = false;

    const ACCEPTED = [
        "video/mp4",
        "video/webm",
        "video/ogg",
        "video/quicktime",
    ];
    const MAX_SIZE = 1024 * 1024 * 1024; // 1GB

    function formatSize(bytes: number): string {
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
        if (bytes < 1024 * 1024 * 1024)
            return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
        return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
    }

    async function handleFile(file: File) {
        errorMessage = "";

        if (!ACCEPTED.includes(file.type)) {
            errorMessage = `Unsupported format. Use MP4, WebM, OGG, or MOV.`;
            return;
        }
        if (file.size > MAX_SIZE) {
            errorMessage = `File too large. Max 1GB.`;
            return;
        }

        state = "uploading";
        progress = 0;

        try {
            // Step 1: Get presigned upload URL from our Rust API
            const init = await initUpload(file.name, file.size, file.type);
            videoId = init.video_id;

            // Step 2: Upload directly to R2 — server never sees the bytes
            await uploadToR2(init.upload_url, file, (pct) => {
                progress = pct;
            });

            // Step 3: Tell backend upload is done
            state = "completing";
            const complete = await completeUpload(videoId);
            watchUrl = `${window.location.origin}/watch/${videoId}`;

            state = "done";
        } catch (err: any) {
            errorMessage = err.message ?? "Upload failed. Please try again.";
            state = "error";
        }
    }

    function handleDrop(e: DragEvent) {
        e.preventDefault();
        isDragging = false;
        const file = e.dataTransfer?.files[0];
        if (file) handleFile(file);
    }

    function handleInput(e: Event) {
        const input = e.target as HTMLInputElement;
        const file = input.files?.[0];
        if (file) handleFile(file);
    }

    function handleDragOver(e: DragEvent) {
        e.preventDefault();
        isDragging = true;
    }

    function reset() {
        state = "idle";
        progress = 0;
        watchUrl = "";
        videoId = "";
        errorMessage = "";
    }

    async function copyLink() {
        await navigator.clipboard.writeText(watchUrl);
        copied = true;
        setTimeout(() => (copied = false), 2000);
    }

    let copied = false;
</script>

<svelte:head>
    <title>StreamVault — Upload</title>
</svelte:head>

<main>
    <header>
        <div class="logo">
            <span class="logo-mark">▶</span>
            <span class="logo-text"
                >STREAM<span class="accent">VAULT</span></span
            >
        </div>
        <p class="tagline">upload once. stream instantly.</p>
    </header>

    <section class="upload-area">
        {#if state === "idle" || state === "error"}
            <!-- Drop zone -->
            <label
                class="dropzone"
                class:dragging={isDragging}
                on:dragover={handleDragOver}
                on:dragleave={() => (isDragging = false)}
                on:drop={handleDrop}
            >
                <input
                    type="file"
                    accept="video/mp4,video/webm,video/ogg,video/quicktime,.mp4,.webm,.ogg,.mov"
                    on:change={handleInput}
                    hidden
                />

                <div class="dropzone-inner">
                    <div class="upload-icon">
                        <svg
                            width="48"
                            height="48"
                            viewBox="0 0 48 48"
                            fill="none"
                        >
                            <path
                                d="M24 8L24 32M24 8L16 16M24 8L32 16"
                                stroke="currentColor"
                                stroke-width="2.5"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            />
                            <path
                                d="M8 36H40"
                                stroke="currentColor"
                                stroke-width="2.5"
                                stroke-linecap="round"
                            />
                        </svg>
                    </div>
                    <p class="drop-primary">Tap to select a video</p>
                    <p class="drop-secondary">or drag and drop</p>
                    <div class="formats">
                        <span>MP4</span><span>WebM</span><span>MOV</span><span
                            >OGG</span
                        >
                    </div>
                    <p class="size-limit">
                        Up to 1 GB · Anonymous · No account needed
                    </p>
                </div>
            </label>

            {#if errorMessage}
                <div class="error-banner">
                    <span class="error-icon">!</span>
                    {errorMessage}
                </div>
            {/if}
        {:else if state === "uploading"}
            <!-- Progress -->
            <div class="progress-panel">
                <div class="progress-header">
                    <span class="progress-label">UPLOADING</span>
                    <span class="progress-pct font-mono">{progress}%</span>
                </div>
                <div class="progress-track">
                    <div class="progress-fill" style="width: {progress}%"></div>
                </div>
                <p class="progress-sub">
                    Sending directly to storage — no server in the way
                </p>
            </div>
        {:else if state === "completing"}
            <div class="progress-panel">
                <div class="progress-header">
                    <span class="progress-label">FINALIZING</span>
                    <span class="spinner">◌</span>
                </div>
                <div class="progress-track">
                    <div class="progress-fill" style="width: 100%"></div>
                </div>
                <p class="progress-sub">Registering your video...</p>
            </div>
        {:else if state === "done"}
            <!-- Success -->
            <div class="success-panel">
                <div class="success-icon">✓</div>
                <h2 class="success-title">Ready to stream</h2>
                <p class="success-sub">
                    Your video is live. Share the link below.
                </p>

                <div class="link-box">
                    <span class="link-text font-mono">{watchUrl}</span>
                    <button class="copy-btn" on:click={copyLink}>
                        {copied ? "COPIED!" : "COPY"}
                    </button>
                </div>

                <div class="action-row">
                    <a class="watch-btn" href="/watch/{videoId}">
                        <span>▶</span> Watch now
                    </a>
                    <button class="upload-another" on:click={reset}>
                        Upload another
                    </button>
                </div>
            </div>
        {/if}
    </section>

    <footer>
        <span
            >No account needed · Videos stored privately · Instant streaming</span
        >
    </footer>
</main>

<style>
    main {
        min-height: 100vh;
        width: 100%;
        max-width: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 1.5rem 1rem;
        gap: 2rem;
        position: relative;
        overflow: hidden;
    }

    main::before {
        content: "";
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: radial-gradient(
                ellipse 60% 40% at 20% 20%,
                rgba(232, 255, 0, 0.04) 0%,
                transparent 70%
            ),
            radial-gradient(
                ellipse 40% 60% at 80% 80%,
                rgba(232, 255, 0, 0.03) 0%,
                transparent 70%
            );
        pointer-events: none;
        z-index: 0;
    }

    /* Header */
    header {
        text-align: center;
    }

    .logo {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        justify-content: center;
        margin-bottom: 0.5rem;
    }

    .logo-mark {
        font-size: 1.5rem;
        color: var(--accent);
    }

    .logo-text {
        font-family: var(--sans);
        font-size: 1.5rem;
        font-weight: 800;
        letter-spacing: 0.15em;
    }

    .accent {
        color: var(--accent);
    }

    .tagline {
        font-family: var(--mono);
        font-size: 0.7rem;
        color: var(--text-muted);
        letter-spacing: 0.1em;
        text-transform: uppercase;
    }

    /* Upload area */
    .upload-area {
        width: 100%;
        max-width: 560px;
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    /* Drop zone */
    .dropzone {
        display: block;
        border: 1.5px solid var(--border);
        background: var(--surface);
        cursor: pointer;
        transition:
            border-color 0.15s,
            background 0.15s;
        position: relative;
        overflow: hidden;
        /* Larger tap target on mobile */
        -webkit-tap-highlight-color: transparent;
    }

    .dropzone::before {
        content: "";
        position: absolute;
        inset: 0;
        background: linear-gradient(
            135deg,
            transparent 40%,
            rgba(232, 255, 0, 0.02) 100%
        );
        pointer-events: none;
    }

    .dropzone:hover,
    .dropzone.dragging {
        border-color: var(--accent);
        background: #151500;
    }

    .dropzone-inner {
        padding: 2rem 1.5rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.5rem;
        text-align: center;
    }

    .upload-icon {
        color: var(--text-muted);
        margin-bottom: 0.5rem;
        transition: color 0.15s;
    }

    .dropzone:hover .upload-icon {
        color: var(--accent);
    }

    .drop-primary {
        font-family: var(--sans);
        font-weight: 700;
        font-size: 1.1rem;
        color: var(--text);
    }

    .drop-secondary {
        font-family: var(--mono);
        font-size: 0.75rem;
        color: var(--text-muted);
    }

    .formats {
        display: flex;
        gap: 0.5rem;
        margin-top: 0.75rem;
        flex-wrap: wrap;
        justify-content: center;
    }

    .formats span {
        font-family: var(--mono);
        font-size: 0.65rem;
        letter-spacing: 0.05em;
        padding: 0.25rem 0.5rem;
        border: 1px solid var(--border);
        color: var(--text-muted);
    }

    .size-limit {
        font-family: var(--mono);
        font-size: 0.65rem;
        color: var(--text-muted);
        margin-top: 0.5rem;
    }

    /* Error */
    .error-banner {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.875rem 1rem;
        border: 1px solid var(--danger);
        background: rgba(255, 68, 68, 0.08);
        font-family: var(--mono);
        font-size: 0.8rem;
        color: var(--danger);
    }

    .error-icon {
        width: 20px;
        height: 20px;
        border: 1.5px solid var(--danger);
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 0.7rem;
        flex-shrink: 0;
    }

    /* Progress */
    .progress-panel {
        padding: 2rem 1.5rem;
        border: 1.5px solid var(--border);
        background: var(--surface);
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .progress-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .progress-label {
        font-family: var(--mono);
        font-size: 0.7rem;
        letter-spacing: 0.15em;
        color: var(--accent);
    }

    .progress-pct {
        font-family: var(--mono);
        font-size: 1.5rem;
        font-weight: 700;
        color: var(--text);
    }

    .progress-track {
        height: 2px;
        background: var(--border);
        position: relative;
        overflow: hidden;
    }

    .progress-fill {
        height: 100%;
        background: var(--accent);
        transition: width 0.2s ease;
        position: relative;
    }

    .progress-fill::after {
        content: "";
        position: absolute;
        right: 0;
        top: -2px;
        width: 4px;
        height: 6px;
        background: var(--accent);
        box-shadow: 0 0 8px var(--accent);
    }

    .progress-sub {
        font-family: var(--mono);
        font-size: 0.7rem;
        color: var(--text-muted);
    }

    .spinner {
        font-size: 1.5rem;
        animation: spin 1.5s linear infinite;
        display: inline-block;
        color: var(--accent);
    }

    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }

    /* Success */
    .success-panel {
        padding: 2rem 1.5rem;
        border: 1.5px solid var(--accent);
        background: var(--surface);
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
        text-align: center;
    }

    .success-icon {
        width: 48px;
        height: 48px;
        border: 2px solid var(--success);
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 1.25rem;
        color: var(--success);
    }

    .success-title {
        font-family: var(--sans);
        font-weight: 800;
        font-size: 1.5rem;
    }

    .success-sub {
        font-family: var(--mono);
        font-size: 0.75rem;
        color: var(--text-muted);
    }

    .link-box {
        width: 100%;
        display: flex;
        border: 1px solid var(--border);
        overflow: hidden;
        margin-top: 0.5rem;
    }

    .link-text {
        flex: 1;
        padding: 0.75rem 1rem;
        font-size: 0.7rem;
        color: var(--text-muted);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        font-family: var(--mono);
    }

    .copy-btn {
        padding: 0.75rem 1.25rem;
        background: var(--border);
        color: var(--text);
        font-size: 0.65rem;
        letter-spacing: 0.1em;
        transition:
            background 0.15s,
            color 0.15s;
        flex-shrink: 0;
        font-family: var(--mono);
        /* Minimum tap target */
        min-height: 44px;
    }

    .copy-btn:hover {
        background: var(--accent);
        color: var(--bg);
    }

    .action-row {
        display: flex;
        gap: 0.75rem;
        width: 100%;
        margin-top: 0.5rem;
        /* Stack vertically on very small screens */
        flex-wrap: wrap;
    }

    .watch-btn {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.875rem;
        background: var(--accent);
        color: var(--bg);
        font-family: var(--mono);
        font-size: 0.8rem;
        font-weight: 700;
        letter-spacing: 0.05em;
        transition: opacity 0.15s;
        /* Minimum tap target */
        min-height: 44px;
    }

    .watch-btn:hover {
        opacity: 0.85;
    }

    .upload-another {
        padding: 0.875rem 1.25rem;
        background: transparent;
        border: 1px solid var(--border);
        color: var(--text-muted);
        font-size: 0.75rem;
        letter-spacing: 0.05em;
        transition:
            border-color 0.15s,
            color 0.15s;
        font-family: var(--mono);
        /* Minimum tap target */
        min-height: 44px;
    }

    .upload-another:hover {
        border-color: var(--text-muted);
        color: var(--text);
    }

    /* Footer */
    footer {
        font-family: var(--mono);
        font-size: 0.65rem;
        color: var(--text-muted);
        letter-spacing: 0.05em;
        text-align: center;
    }

    /* ── Mobile breakpoint ───────────────────────────────────────────────────── */
    @media (max-width: 480px) {
        main {
            padding: 1.25rem 0.875rem;
            gap: 1.5rem;
            justify-content: flex-start;
            padding-top: 2rem;
        }

        .logo-text {
            font-size: 1.25rem;
        }

        .dropzone-inner {
            padding: 1.75rem 1rem;
        }

        .drop-primary {
            font-size: 1rem;
        }

        /* On mobile there is no drag and drop — change the label */
        .drop-secondary {
            display: none;
        }

        .action-row {
            flex-direction: column;
        }

        .watch-btn,
        .upload-another {
            width: 100%;
            min-height: 48px;
        }

        .success-title {
            font-size: 1.25rem;
        }
    }
</style>
