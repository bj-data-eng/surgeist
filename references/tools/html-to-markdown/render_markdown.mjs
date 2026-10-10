import fs from 'node:fs';
const { marked } = await import(process.env.MARKED_MODULE || 'marked');
const work=process.argv[2]||'tmp/work/cssom-reference-conversion';
const r=JSON.parse(fs.readFileSync(work+'/conversion-record.json','utf8'));
const md=fs.readFileSync('references/'+r.filename,'utf8');
fs.writeFileSync(work+'/rendered.html', marked.parse(md,{gfm:true}));
fs.writeFileSync(work+'/rendered-body.html', marked.parse(md.split('<!-- captured-body-start -->')[1],{gfm:true}));
