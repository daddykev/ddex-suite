// packages/ddex-parser/tests/integration/npm-package.test.ts
import * as fs from 'fs';
import * as path from 'path';

describe('NPM Package Structure', () => {
  const bindingsPath = path.join(__dirname, '../../bindings/node');
  
  test('package.json is valid', () => {
    const pkgPath = path.join(bindingsPath, 'package.json');
    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'));
    
    // Essential fields
    expect(pkg.name).toBe('ddex-parser');
    expect(pkg.version).toMatch(/^\d+\.\d+\.\d+/);
    expect(pkg.main).toBeDefined();
    expect(pkg.types).toBeDefined();
    expect(pkg.license).toBe('MIT');
    
    // Repository info
    expect(pkg.repository).toBeDefined();
    expect(pkg.repository.url).toContain('github.com/daddykev/ddex-suite');
    
    // Files to include
    expect(pkg.files).toContain('dist');
    expect(pkg.files).toContain('README.md');
    
    // Keywords for discoverability
    expect(pkg.keywords).toContain('ddex');
    expect(pkg.keywords).toContain('parser');
  });
  
  test('README exists and is comprehensive', () => {
    const readmePath = path.join(bindingsPath, 'README.md');
    expect(fs.existsSync(readmePath)).toBe(true);
    
    const readme = fs.readFileSync(readmePath, 'utf-8');
    expect(readme.length).toBeGreaterThan(1000);
    
    // Check for essential sections
    expect(readme).toContain('npm install ddex-parser');
    expect(readme).toContain('Usage');
    expect(readme).toContain('Features');
    expect(readme).toContain('API');
  });
  
  test('TypeScript definitions exist', () => {
    const pkg = JSON.parse(
      fs.readFileSync(path.join(bindingsPath, 'package.json'), 'utf-8')
    );
    
    if (pkg.types) {
      const typesPath = path.join(bindingsPath, pkg.types);
      expect(fs.existsSync(typesPath)).toBe(true);
    }
  });
  
  test('no source files in package', () => {
    const files = fs.readdirSync(bindingsPath);
    
    // These should NOT be in the published package
    expect(files).not.toContain('src');
    expect(files).not.toContain('tsconfig.json');
    expect(files).not.toContain('.env');
  });
});