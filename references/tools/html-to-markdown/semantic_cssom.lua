local function escape(s)
  return (s:gsub('[&<>"\\*_`%[%]#|~:]', function(c) return '&#' .. string.byte(c) .. ';' end))
end
local function idinline(id)
  return pandoc.RawInline('html','<a id="'..escape(id)..'"></a>')
end
local function prefix(el)
  local result={}
  if el.identifier and el.identifier~='' then table.insert(result,idinline(el.identifier)) end
  return result
end
local function append(a,b)
  for _,v in ipairs(b) do table.insert(a,v) end
  return a
end
function Header(el)
  local p={}
  el.identifier='';el.classes={};el.attributes={}
  el.content=append(p,el.content)
  return el
end
function Span(el)
  local p=prefix(el)
  local isvar=false
  local iscode=false
  for _,v in ipairs(el.classes) do
    if v=='source-var' then isvar=true end
    if v=='source-linked-code' then iscode=true end
  end
  if iscode then
    table.insert(p,pandoc.RawInline('html','<code>'))
    append(p,el.content)
    table.insert(p,pandoc.RawInline('html','</code>'))
    return p
  end
  if isvar then
    table.insert(p,pandoc.RawInline('html','<var>'))
    append(p,el.content)
    table.insert(p,pandoc.RawInline('html','</var>'))
    return p
  end
  return append(p,el.content)
end
function Div(el)
  local p={}
  if el.identifier~='' then table.insert(p,pandoc.RawBlock('html','<a id="'..escape(el.identifier)..'"></a>')) end
  return append(p,el.content)
end
function Link(el)
  local p=prefix(el)
  el.identifier='';el.classes={};el.attributes={};el.title=''
  table.insert(p,el)
  return p
end
function Image(el)
  local p=prefix(el)
  el.identifier='';el.classes={};el.attributes={};el.title=''
  table.insert(p,el)
  return p
end
function Code(el)
  local p=prefix(el)
  table.insert(p,pandoc.RawInline('html','<code>'..escape(el.text)..'</code>'))
  return p
end
function CodeBlock(el)
  el.text=el.text:gsub('^surgeist%-preserve%-leading%-newline','')
  el.classes={'text'};el.attributes={}
  return el
end
function Str(el)
  if el.text:find('@',1,true) then
    return pandoc.RawInline('html',escape(el.text):gsub('@','&#64;'))
  end
end
function Emph(el)
  return append(append({pandoc.RawInline('html','<em>')},el.content),{pandoc.RawInline('html','</em>')})
end
function Strong(el)
  return append(append({pandoc.RawInline('html','<strong>')},el.content),{pandoc.RawInline('html','</strong>')})
end
function Superscript(el)
  return append(append({pandoc.RawInline('html','<sup>')},el.content),{pandoc.RawInline('html','</sup>')})
end
function Subscript(el)
  return append(append({pandoc.RawInline('html','<sub>')},el.content),{pandoc.RawInline('html','</sub>')})
end
