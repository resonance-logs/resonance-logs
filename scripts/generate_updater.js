import fs from 'fs';
import path from 'path';

const args = process.argv.slice(2);
if (args.length < 4) {
    console.error('Usage: node generate_updater.js <version> <notes_file> <bundle_dir> <output_file>');
    process.exit(1);
}

const [version, notesFile, bundleDir, outputFile] = args;

let notes = "";
try {
    notes = fs.readFileSync(notesFile, 'utf8');
} catch {
    console.warn(`Could not read notes file: ${notesFile}`);
    notes = "Release " + version;
}

const pub_date = new Date().toISOString();

function findFiles(dir) {
    let results = [];
    if (!fs.existsSync(dir)) return results;
    const list = fs.readdirSync(dir);
    list.forEach(file => {
        file = path.join(dir, file);
        const stat = fs.statSync(file);
        if (stat && stat.isDirectory()) {
            results = results.concat(findFiles(file));
        } else {
            results.push(file);
        }
    });
    return results;
}

const allFiles = findFiles(bundleDir);
console.log(`Found ${allFiles.length} files in ${bundleDir}`);

const msiFile = allFiles.find(f => f.endsWith('.msi'));
const nsisFile = allFiles.find(f => f.endsWith('.exe') && !f.includes('alts')); // Exclude partials if any

const platforms = {};

if (msiFile) {
    const sigPath = msiFile + '.sig';
    if (fs.existsSync(sigPath)) {
        const signature = fs.readFileSync(sigPath, 'utf8');
        const filename = path.basename(msiFile);
        const url = `https://github.com/resonance-logs/resonance-logs/releases/download/v${version}/${filename}`;

        platforms['windows-x86_64'] = { signature, url };
        platforms['windows-x86_64-msi'] = { signature, url };
        console.log(`Added MSI: ${filename}`);
    } else {
        console.warn(`Signature missing for MSI: ${sigPath}`);
    }
}

if (nsisFile) {
    const sigPath = nsisFile + '.sig';
    if (fs.existsSync(sigPath)) {
        const signature = fs.readFileSync(sigPath, 'utf8');
        const filename = path.basename(nsisFile);
        const url = `https://github.com/resonance-logs/resonance-logs/releases/download/v${version}/${filename}`;

        platforms['windows-x86_64-nsis'] = { signature, url };
        if (!platforms['windows-x86_64']) {
             platforms['windows-x86_64'] = { signature, url };
        }
        console.log(`Added NSIS: ${filename}`);
    } else {
        console.warn(`Signature missing for NSIS: ${sigPath}`);
    }
}

const updateData = {
    version,
    notes,
    pub_date,
    platforms
};

fs.writeFileSync(outputFile, JSON.stringify(updateData, null, 2));
console.log(`Generated ${outputFile}`);
