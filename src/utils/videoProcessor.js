import { invoke, convertFileSrc } from '@tauri-apps/api/core';

export async function processVideoForAI(id, location) {
    return new Promise((resolve) => {
        const video = document.createElement('video');
        video.muted = true;
        video.playsInline = true;
        video.crossOrigin = 'anonymous';

        // Same asset logic as PhotoViewer
        let src = convertFileSrc(location);
        if (src === location && (src.startsWith('/') || /^[a-zA-Z]:\\/.test(src))) {
            const path = src.startsWith('/') ? src : '/' + src.replace(/\\/g, '/');
            src = `https://asset.localhost${path}`;
        }
        
        video.src = src;

        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        const frames = [];
        let thumbnail = null;

        video.addEventListener('loadeddata', async () => {
            // Resize canvas to max 400px to match thumbnail/AI requirements
            const maxDimension = 400;
            let width = video.videoWidth;
            let height = video.videoHeight;
            
            if (width > height && width > maxDimension) {
                height *= maxDimension / width;
                width = maxDimension;
            } else if (height > maxDimension) {
                width *= maxDimension / height;
                height = maxDimension;
            }
            
            canvas.width = width;
            canvas.height = height;

            const duration = video.duration || 10;
            const captureTimes = [1, Math.min(5, duration / 2), Math.min(10, duration * 0.9)];

            for (let i = 0; i < captureTimes.length; i++) {
                const time = captureTimes[i];
                video.currentTime = time;
                
                await new Promise(r => {
                    const onSeeked = () => {
                        video.removeEventListener('seeked', onSeeked);
                        r();
                    };
                    video.addEventListener('seeked', onSeeked);
                });

                ctx.drawImage(video, 0, 0, width, height);
                const b64 = canvas.toDataURL('image/jpeg', 0.8);
                
                if (i === 0) thumbnail = b64;
                frames.push(b64);
            }

            // Send thumbnail to backend
            await invoke('update_video_thumbnail', { id, encoded: thumbnail });
            
            // Send frames to ML worker
            await invoke('process_video_frames', { id, frames });
            
            video.remove();
            canvas.remove();
            resolve(thumbnail);
        });

        video.addEventListener('error', () => {
            console.error('Failed to load video for processing:', location);
            video.remove();
            canvas.remove();
            resolve(null);
        });
        
        // Trigger load
        video.load();
    });
}
