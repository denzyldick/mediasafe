const express = require('express');
const fs = require('fs');
const path = require('path');
const app = express();
const PORT = 80;

app.use(express.json());
app.use(express.static(path.join(__dirname)));

const WAITLIST_FILE = path.join(__dirname, 'data', 'waitlist.csv');

// Ensure data directory exists
if (!fs.existsSync(path.join(__dirname, 'data'))) {
    fs.mkdirSync(path.join(__dirname, 'data'));
}

// Ensure CSV exists with headers
if (!fs.existsSync(WAITLIST_FILE)) {
    fs.writeFileSync(WAITLIST_FILE, 'timestamp,email,os\n');
}

app.post('/api/waitlist', (req, res) => {
    const { email, os } = req.body;
    if (!email) return res.status(400).send('Email is required');

    const timestamp = new Date().toISOString();
    const row = `${timestamp},${email},${os || 'unknown'}\n`;

    fs.appendFile(WAITLIST_FILE, row, (err) => {
        if (err) {
            console.error('Failed to save to waitlist:', err);
            return res.status(500).send('Internal server error');
        }
        console.log(`New signup: ${email} (${os})`);
        res.status(200).send('Successfully joined the waitlist');
    });
});

app.listen(PORT, '0.0.0.0', () => {
    console.log(`Siegu Landing Page Server running on port ${PORT}`);
});
