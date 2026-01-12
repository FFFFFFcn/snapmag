import sharp from 'sharp';
import path from 'path';
import { fileURLToPath } from 'url';
import { exec } from 'child_process';
import { promisify } from 'util';
import fs from 'fs';
import pngToIco from 'png-to-ico';

const execAsync = promisify(exec);

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 源图标路径
const sourceIcon = 'C:\\Users\\ZIFENG ZHONG\\Downloads\\image-u2MCIEUGSMFedZ5TUCIlWtZNH7wcfY.png';
// 图标输出目录
const iconsDir = path.join(__dirname, 'src-tauri', 'icons');

// 需要生成的图标尺寸
const iconSizes = [
  { size: 32, name: '32x32.png' },
  { size: 64, name: '64x64.png' },
  { size: 128, name: '128x128.png' },
  { size: 256, name: '256x256.png' }, // 任务栏图标需要 256x256
  { size: 128, name: '128x128@2x.png' }, // 视网膜屏幕使用
  { size: 256, name: '256x256@2x.png' }, // 高 DPI 视网膜屏幕使用
  { size: 64, name: 'tray-icon.png' }, // 托盘图标 - 使用 64x64 以获得更好的清晰度
];

// 生成图标
async function generateIcons() {
  try {
    console.log('开始生成图标...');
    
    for (const { size, name } of iconSizes) {
      const outputPath = path.join(iconsDir, name);
      await sharp(sourceIcon)
        .resize(size, size, {
          fit: 'cover',
          position: 'center',
        })
        .png()
        .toFile(outputPath);
      console.log(`✓ 生成 ${name} (${size}x${size})`);
    }
    
    // 生成 Windows .ico 文件
    console.log('生成 Windows .ico 文件...');
    const icoPath = path.join(iconsDir, 'icon.ico');
    
    // 生成多个尺寸的 PNG 用于 .ico 文件
    const icoSizes = [16, 32, 48, 64, 128, 256];
    const icoBuffers = [];
    
    for (const size of icoSizes) {
      const buffer = await sharp(sourceIcon)
        .resize(size, size, { fit: 'cover', position: 'center' })
        .png()
        .toBuffer();
      icoBuffers.push(buffer);
    }
    
    const icoBuffer = await pngToIco(icoBuffers);
    fs.writeFileSync(icoPath, icoBuffer);
    console.log('✓ 生成 icon.ico');
    
    // 生成 macOS .icns 文件（需要 iconutil 工具，仅在 macOS 上可用）
    if (process.platform === 'darwin') {
      console.log('生成 macOS .icns 文件...');
      const iconsetDir = path.join(iconsDir, 'icon.iconset');
      
      // 创建 iconset 目录
      await execAsync(`mkdir -p "${iconsetDir}"`);
      
      // 生成不同尺寸的图标
      const sizes = [16, 32, 64, 128, 256, 512, 1024];
      for (const size of sizes) {
        const normalPath = path.join(iconsetDir, `icon_${size}x${size}.png`);
        const retinaPath = path.join(iconsetDir, `icon_${size}x${size}@2x.png`);
        
        await sharp(sourceIcon)
          .resize(size, size, { fit: 'cover', position: 'center' })
          .png()
          .toFile(normalPath);
        
        await sharp(sourceIcon)
          .resize(size * 2, size * 2, { fit: 'cover', position: 'center' })
          .png()
          .toFile(retinaPath);
      }
      
      // 使用 iconutil 生成 .icns 文件
      await execAsync(`iconutil -c icns "${iconsetDir}" -o "${path.join(iconsDir, 'icon.icns')}"`);
      console.log('✓ 生成 icon.icns');
    } else {
      console.log('⚠ 跳过 macOS .icns 文件生成（仅在 macOS 上可用）');
    }
    
    console.log('图标生成完成！');
  } catch (error) {
    console.error('生成图标失败:', error);
    process.exit(1);
  }
}

generateIcons();
