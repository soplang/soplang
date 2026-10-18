use std::{collections::HashMap,path::{Path,PathBuf}};
use soplang::{analysis::{check_source,overlay_loader,Limits,index::Index},frontend::trivia::lossless_tokens};
#[test]
fn editor_analysis_uses_unsaved_imports_without_execution(){
 let mut overlays=HashMap::new();overlays.insert(PathBuf::from("/tmp/editor-library.sop"),"qor(gelin())\nhawl add(abn a, abn b): abn { celi a+b }".to_owned());
 let result=check_source("keen \"editor-library.sop\"\nqor(add(1,2))",Path::new("/tmp/editor-main.sop"),false,Limits::default(),&|p|overlay_loader(&overlays,p),&||false);
 assert!(result.error.is_none(),"{:?}",result.error);
 let index=Index::build(&result.files);let symbol=index.symbols.iter().find(|s|s.name=="add").unwrap();assert!(symbol.location.path.ends_with("editor-library.sop"));assert_eq!(index.occurrences.iter().filter(|o|o.symbol==symbol.id).count(),2);
}
#[test]
fn editor_analysis_preserves_file_redeclaration_and_strict_rules(){
 for(source,strict,code)in[("door x=1 door x=2",false,"E020"),("door x=1",true,"E022")]{let result=check_source(source,Path::new("/tmp/rules.sop"),strict,Limits::default(),&|_|unreachable!(),&||false);assert_eq!(result.error.unwrap().code(),Some(code));}
}
#[test]
fn lossless_source_pieces_cover_every_byte(){let source="// heading\r\nqoraal x = 'a+b https://host/😀' /* comment */\nqor(x == 'C:\\work')\n";let pieces=lossless_tokens(source).unwrap();let mut cursor=0;let mut restored=String::new();for piece in pieces{assert_eq!(piece.range.start,cursor);restored.push_str(&source[piece.range.clone()]);cursor=piece.range.end;}assert_eq!(cursor,source.len());assert_eq!(restored,source);}
#[test]
fn rename_simulation_rejects_reverse_capture(){let result=check_source("abn other=1 hawl outer(abn value): abn { door f=hawl(): abn { celi other } celi value+f() }",Path::new("/tmp/capture.sop"),false,Limits::default(),&|_|unreachable!(),&||false);assert!(result.error.is_none());let index=Index::build(&result.files);let symbol=index.symbols.iter().find(|s|s.name=="value").unwrap();assert!(!index.rename_preserves_bindings(&symbol.id,"other"));assert!(index.rename_preserves_bindings(&symbol.id,"renamed"));}
