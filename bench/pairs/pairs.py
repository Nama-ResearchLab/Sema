import json,subprocess,csv,re,collections
def V(s,t,n,o,r,d='-',m='a'): return "{v:%s|%s|%s|%s|%s|%s|%s}"%(s,t,'neg' if n else '-',o,r,d,m)
T=lambda x:"{t:%s}"%x
P=[]  # (cat, swahili input, input_negated, illustrative model reply (EN), template, gold swahili)
def a(cat,sw,neg,en,tpl,gold): P.append((cat,sw,neg,en,tpl,gold))
IMP_GIVE=V('-','-',0,'m','pa','-','e')
# --- dosing ---
a('dosing',"Nimpe mtoto dawa kiasi gani?",0,"Give the child one tablet twice a day.",f"{IMP_GIVE} {T('child')} {T('one tablet')} {T('twice a day')}.","Mpe mtoto kidonge kimoja mara mbili kwa siku.")
a('dosing',"Dawa hii nitumie lini?",0,"Use the medicine after meals.",f"{V('-','-',0,'-','tumia')} {T('medicine')} {T('after meals')}.","Tumia dawa baada ya kula.")
a('dosing',"Nimeze vidonge vingapi?",0,"Swallow two tablets every morning.",f"{V('-','-',0,'-','meza')} {T('two tablets')} {T('every morning')}.","Meza vidonge viwili kila asubuhi.")
a('dosing',"Nitumie dawa kwa siku ngapi?",0,"Use the medicine for three days.",f"{V('-','-',0,'-','tumia')} {T('medicine')} kwa {T('three days')}.","Tumia dawa kwa siku tatu.")
a('dosing',"Mtoto anaharisha, nimpe nini?",0,"Give the child oral rehydration solution.",f"{IMP_GIVE} {T('child')} {T('oral rehydration solution')}.","Mpe mtoto maji ya chumvi na sukari.")
a('dosing',"Dawa niitumie kabla ya kula?",0,"Use the medicine before meals.",f"{V('-','-',0,'-','tumia')} {T('medicine')} {T('before meals')}.","Tumia dawa kabla ya kula.")
a('dosing',"Nirudi lini kliniki?",0,"Return to the clinic after one week.",f"{T('return to the clinic')} baada ya {T('one week')}.","Rudi kliniki baada ya wiki moja.")
a('dosing',"Mtoto anywe maji kiasi gani?",0,"Give the child plenty of water.",f"{IMP_GIVE} {T('child')} {T('water')} mengi.","Mpe mtoto maji mengi.")
a('dosing',"Nipake dawa ya ngozi mara ngapi?",0,"Apply the medicine twice a day.",f"{V('-','-',0,'-','paka')} {T('medicine')} {T('twice a day')}.","Paka dawa mara mbili kwa siku.")
a('dosing',"Dawa ya malaria nimeze lini?",0,"Swallow the medicine every morning.",f"{V('-','-',0,'-','meza')} {T('medicine')} {T('every morning')}.","Meza dawa kila asubuhi.")
a('dosing',"Mama mjamzito anywe nini?",0,"Give the pregnant woman water.",f"{IMP_GIVE} {T('pregnant woman')} {T('water')}.","Mpe mama mjamzito maji.")
a('dosing',"Nimeze kidonge kimoja au viwili?",0,"Swallow one tablet.",f"{V('-','-',0,'-','meza')} {T('one tablet')}.","Meza kidonge kimoja.")
a('dosing',"Dawa ya kikohozi nimpe mtoto mara ngapi?",0,"Give the child the medicine three times a day.",f"{IMP_GIVE} {T('child')} {T('medicine')} {T('three times a day')}.","Mpe mtoto dawa mara tatu kwa siku.")
a('dosing',"Nipime nini kwanza?",0,"Measure the blood pressure.",f"{V('-','-',0,'-','pima')} {T('blood pressure')}.","Pima shinikizo la damu.")
a('dosing',"Nimchukue wapi mgonjwa?",0,"Take the patient to the hospital.",f"{V('-','-',0,'m','peleka','-','e')} {T('patient')} {T('hospital')}.","Mpeleke mgonjwa hospitali.")
# --- negative advice ---
a('negative_advice',"Nimpe mtoto dawa hii?",0,"Do not give the child this medicine.",f"{V('2SG','-',1,'m','pa','-','e')} {T('child')} {T('medicine')} hii.","Usimpe mtoto dawa hii.")
a('negative_advice',"Mtoto anywe maji haya?",0,"The child should not drink this water.",f"{T('child')} {V('3SG','-',1,'-','nywa','-','e')} {T('water')} haya.","Mtoto asinywe maji haya.")
a('negative_advice',"Nile chakula baridi?",0,"Do not eat cold food.",f"{V('2SG','-',1,'-','la','-','e')} {T('food')} baridi.","Usile chakula baridi.")
a('negative_advice',"Nitumie dawa gani?",0,"Do not use aspirin.",f"{V('2SG','-',1,'-','tumia','-','e')} {T('aspirin')}.","Usitumie aspirini.")
a('negative_advice',"Mgonjwa apewe nini?",0,"The patient should not be given aspirin.",f"{T('patient')} {V('3SG','-',1,'-','pa','passive','e')} {T('aspirin')}.","Mgonjwa asipewe aspirini.")
a('negative_advice',"Hawezi kunywa maji",1,"Take the patient to the hospital.",f"{V('-','-',0,'m','peleka','-','e')} {T('patient')} {T('hospital')}.","Mpeleke mgonjwa hospitali.")
a('negative_advice',"Mtoto hajala tangu jana",1,"Give the child food and water.",f"{IMP_GIVE} {T('child')} {T('food')} na {T('water')}.","Mpe mtoto chakula na maji.")
a('negative_advice',"Sijapata nafuu",1,"Return to the clinic after one week.",f"{T('return to the clinic')} baada ya {T('one week')}.","Rudi kliniki baada ya wiki moja.")
a('negative_advice',"Hatuna dawa nyumbani",1,"Go to the pharmacy.",f"{V('-','-',0,'-','enda')} {T('pharmacy')}.","Nenda duka la dawa.")
a('negative_advice',"Hajisikii vizuri",1,"Rest in bed.",f"{V('-','-',0,'-','pumzika')} kitandani.","Pumzika kitandani.")
a('negative_advice',"Tule nini?",0,"Do not eat cold food.",f"{V('2PL','-',1,'-','la','-','e')} {T('food')} baridi.","Msile chakula baridi.")
a('negative_advice',"Tunywe maji gani?",0,"We should not drink dirty water.",f"{V('1PL','-',1,'-','nywa','-','e')} {T('water')} machafu.","Tusinywe maji machafu.")
a('negative_advice',"Mgonjwa atoke nje?",0,"The patient should not go out.",f"{T('patient')} {V('3SG','-',1,'-','toka','-','e')} nje.","Mgonjwa asitoke nje.")
a('negative_advice',"Nimpe mtoto mchanga maji?",0,"Do not give the baby cold water.",f"{V('2SG','-',1,'m','pa','-','e')} {T('baby')} {T('water')} baridi.","Usimpe mtoto mchanga maji baridi.")
a('negative_advice',"Mtoto ale nini?",0,"The child should eat food.",f"{T('child')} {V('3SG','-',0,'-','la','-','e')} {T('food')}.","Mtoto ale chakula.")
# --- follow-up statements ---
a('followup',"Mgonjwa anatumia dawa gani?",0,"The patient is using the medicine.",f"{T('patient')} {V('3SG','PRES',0,'-','tumia')} {T('medicine')}.","Mgonjwa anatumia dawa.")
a('followup',"Mtoto amepata chanjo?",0,"The child has received the vaccine.",f"{T('child')} {V('3SG','PERF',0,'-','pata')} {T('vaccine')}.","Mtoto amepata chanjo.")
a('followup',"Mtoto amelala?",0,"The child slept at night.",f"{T('child')} {V('3SG','PAST',0,'-','lala')} {T('night')}.","Mtoto alilala usiku.")
a('followup',"Daktari amefika?",0,"The doctor has arrived in the morning.",f"{T('doctor')} {V('3SG','PERF',0,'-','fika')} {T('morning')}.","Daktari amefika asubuhi.")
a('followup',"Nani anamlea mtoto?",0,"The mother is caring for the child.",f"{T('mother')} {V('3SG','PRES',0,'m','lea')} {T('child')}.","Mama anamlea mtoto.")
a('followup',"Wauguzi wanafika lini?",0,"The nurses will arrive tomorrow.",f"{T('nurses')} {V('3PL','FUT',0,'-','fika')} {T('tomorrow')}.","Wauguzi watafika kesho.")
a('followup',"Walimwona daktari?",0,"They saw the doctor yesterday.",f"{V('3PL','PAST',0,'m','ona')} {T('doctor')} {T('yesterday')}.","Walimwona daktari jana.")
a('followup',"Utakuja lini kliniki?",0,"I will come to the clinic tomorrow.",f"{V('1SG','FUT',0,'-','ja')} {T('clinic')} {T('tomorrow')}.","Nitakuja kliniki kesho.")
a('followup',"Mnapenda dawa hii?",0,"We like this medicine.",f"{V('1PL','PRES',0,'-','penda')} {T('medicine')} hii.","Tunapenda dawa hii.")
a('followup',"Mtoto anakula nini?",0,"The child eats food.",f"{T('child')} {V('3SG','PRES',0,'-','la')} {T('food')}.","Mtoto anakula chakula.")
a('followup',"Unalala wapi?",0,"I sleep at home.",f"{V('1SG','PRES',0,'-','lala')} {T('home')}.","Ninalala nyumbani.")
a('followup',"Mmekunywa maji?",0,"We drank water yesterday.",f"{V('1PL','PAST',0,'-','nywa')} {T('water')} {T('yesterday')}.","Tulikunywa maji jana.")
a('followup',"Mnasubiri nini?",0,"We are waiting for the doctor.",f"{V('1PL','PRES',0,'m','subiri')} {T('doctor')}.","Tunamsubiri daktari.")
a('followup',"Anaumwa na nini?",0,"The patient has fever.",f"{T('patient')} ana {T('fever')}.","Mgonjwa ana homa.")
a('followup',"Walikuja na nani?",0,"They came with the mother yesterday.",f"{V('3PL','PAST',0,'-','ja')} na {T('mother')} {T('yesterday')}.","Walikuja na mama jana.")
# --- danger signs / referral ---
for sw,neg,en,tpl,g in [
("Mtoto ana homa kali na kutapika",0,"Take the child to the hospital today.",f"{V('-','-',0,'m','peleka','-','e')} {T('child')} {T('hospital')} {T('today')}.","Mpeleke mtoto hospitali leo."),
("Anapumua kwa shida",0,"Take the patient to the hospital today.",f"{V('-','-',0,'m','peleka','-','e')} {T('patient')} {T('hospital')} {T('today')}.","Mpeleke mgonjwa hospitali leo."),
("Mama mjamzito anatokwa damu",0,"Take the pregnant woman to the hospital.",f"{V('-','-',0,'m','peleka','-','e')} {T('pregnant woman')} {T('hospital')}.","Mpeleke mama mjamzito hospitali."),
("Mtoto mchanga hanyonyi",1,"Take the baby to the clinic tomorrow.",f"{V('-','-',0,'m','peleka','-','e')} {T('baby')} {T('clinic')} {T('tomorrow')}.","Mpeleke mtoto mchanga kliniki kesho."),
("Amepoteza fahamu",0,"Bring the patient to the hospital.",f"{V('-','-',0,'m','leta','-','e')} {T('patient')} {T('hospital')}.","Mlete mgonjwa hospitali."),
("Mtoto hakunyonya tangu jana",1,"Take the child to the hospital today.",f"{V('-','-',0,'m','peleka','-','e')} {T('child')} {T('hospital')} {T('today')}.","Mpeleke mtoto hospitali leo."),
("Damu inatoka puani",0,"Take the patient to the clinic.",f"{V('-','-',0,'m','peleka','-','e')} {T('patient')} {T('clinic')}.","Mpeleke mgonjwa kliniki."),
("Mtoto hanywi maji",1,"Give the child oral rehydration solution.",f"{IMP_GIVE} {T('child')} {T('oral rehydration solution')}.","Mpe mtoto maji ya chumvi na sukari."),
("Ana kikohozi wiki tatu",0,"Take a urine sample.",f"{V('-','-',0,'-','chukua')} {T('urine sample')}.","Chukua sampuli ya mkojo."),
("Hapumui vizuri",1,"Take the child to the hospital today.",f"{V('-','-',0,'m','peleka','-','e')} {T('child')} {T('hospital')} {T('today')}.","Mpeleke mtoto hospitali leo."),
("Ana maumivu makali ya kifua",0,"Take the patient to the hospital today.",f"{V('-','-',0,'m','peleka','-','e')} {T('patient')} {T('hospital')} {T('today')}.","Mpeleke mgonjwa hospitali leo."),
("Mkojo una damu",0,"Take a urine sample.",f"{V('-','-',0,'-','chukua')} {T('urine sample')}.","Chukua sampuli ya mkojo."),
("Hajakojoa tangu jana",1,"Give the patient water.",f"{IMP_GIVE} {T('patient')} {T('water')}.","Mpe mgonjwa maji."),
("Ana kuharisha damu",0,"Take the child to the clinic today.",f"{V('-','-',0,'m','peleka','-','e')} {T('child')} {T('clinic')} {T('today')}.","Mpeleke mtoto kliniki leo."),
("Anakohoa damu",0,"Take the patient to the hospital today.",f"{V('-','-',0,'m','peleka','-','e')} {T('patient')} {T('hospital')} {T('today')}.","Mpeleke mgonjwa hospitali leo.")]: a('referral',sw,neg,en,tpl,g)
# --- negative statements (reply negative) ---
for sw,en,s,t,r,o,rest,g in [
("Mgonjwa alikula?","The patient did not eat.",'3SG','PAST','la','-',"","Mgonjwa hakula."),
("Mtoto alinywa dawa?","The child did not drink the medicine.",'3SG','PAST','nywa','-',"{t:medicine}","Mtoto hakunywa dawa."),
("Wauguzi wamefika?","The nurses have not yet arrived.",'3PL','PERF','fika','-',"","Wauguzi hawajafika."),
("Daktari atakuja?","The doctor will not come today.",'3SG','FUT','ja','-',"{t:today}","Daktari hatakuja leo."),
("Unapenda dawa?","I do not like medicine.",'1SG','PRES','penda','-',"{t:medicine}","Sipendi dawa."),
("Mnaweza kuja?","We cannot come tomorrow.",'1PL','PRES','weza','-',"kuja {t:tomorrow}","Hatuwezi kuja kesho."),
("Mtoto anakula?","The child does not eat.",'3SG','PRES','la','-',"","Mtoto hali."),
("Umemwona daktari?","I have not yet seen the doctor.",'1SG','PERF','ona','m',"{t:doctor}","Sijamwona daktari."),
("Ulikunywa dawa?","I did not drink the medicine.",'1SG','PAST','nywa','-',"{t:medicine}","Sikunywa dawa."),
("Watoto wanalala?","The children do not sleep.",'3PL','PRES','lala','-',"","Watoto hawalali."),
("Nani atakuja?","They will not come.",'3PL','FUT','ja','-',"","Hawatakuja."),
("Ulimpa mtoto dawa?","I did not give the child the medicine.",'1SG','PAST','pa','m',"{t:child} {t:medicine}","Sikumpa mtoto dawa."),
("Mnaelewa?","We do not understand.",'1PL','PRES','elewa','-',"","Hatuelewi."),
("Anajua?","He does not know.",'3SG','PRES','jua','-',"","Hajui."),
("Alipata nafuu?","He did not get better.",'3SG','PAST','pata','-',"nafuu","Hakupata nafuu.")]:
    subj_noun={"Mgonjwa alikula?":"{t:patient} ","Mtoto alinywa dawa?":"{t:child} ","Wauguzi wamefika?":"{t:nurses} ","Daktari atakuja?":"{t:doctor} ","Mtoto anakula?":"{t:child} ","Watoto wanalala?":"{t:children} "}.get(sw,"")
    a('negative_reply',sw,0,en,f"{subj_noun}{V(s,t,1,o,r)} {rest}".strip()+".",g)
# --- noun-class agreement ---
for sw,en,tpl,g in [
("Kiti kiko wapi?","The chair is broken.",f"{T('chair')} {V('C7','PERF',0,'-','vunja','stative')}.","Kiti kimevunjika."),
("Kitabu kiko wapi?","The book has been lost.",f"{T('book')} {V('C7','PERF',0,'-','potea')}.","Kitabu kimepotea."),
("Mlango uko wapi?","The door is open.",f"{T('door')} {V('C3','PERF',0,'-','fungua','stative')}.","Mlango umefunguka."),
("Madirisha yako wapi?","The windows are broken.",f"{T('windows')} {V('C6','PERF',0,'-','vunja','stative')}.","Madirisha yamevunjika."),
("Matunda yako wapi?","The fruits have run out.",f"{T('fruits')} {V('C6','PERF',0,'-','isha')}.","Matunda yameisha."),
("Dawa ziko wapi?","The medicines have run out.",f"{T('medicines')} {V('C10','PERF',0,'-','isha')}.","Dawa zimeisha."),
("Gari liko wapi?","The car has arrived.",f"{T('car')} {V('C5','PERF',0,'-','fika')}.","Gari limefika."),
("Basi liko wapi?","The bus will arrive tomorrow.",f"{T('bus')} {V('C5','FUT',0,'-','fika')} {T('tomorrow')}.","Basi litafika kesho."),
("Mbwa yuko wapi?","The dog is sleeping.",f"{T('dog')} {V('3SG','PRES',0,'-','lala')}.","Mbwa analala."),
("Simu iko wapi?","The phone has been lost.",f"{T('phone')} {V('C9','PERF',0,'-','potea')}.","Simu imepotea."),
("Mkate uko wapi?","The bread has run out.",f"{T('bread')} {V('C3','PERF',0,'-','isha')}.","Mkate umeisha."),
("Mti uko wapi?","The tree has fallen.",f"{T('tree')} {V('C3','PERF',0,'-','anguka')}.","Mti umeanguka."),
("Shule itafunguliwa lini?","The school will be opened tomorrow.",f"{T('school')} {V('C9','FUT',0,'-','fungua','passive')} {T('tomorrow')}.","Shule itafunguliwa kesho."),
("Mwalimu atafanya nini?","The teacher will teach the children.",f"{T('teacher')} {V('3SG','FUT',0,'-','fundisha')} {T('children')}.","Mwalimu atafundisha watoto."),
("Wanafunzi wanafanya nini?","The students are reading the book.",f"{T('students')} {V('3PL','PRES',0,'-','soma')} {T('book')}.","Wanafunzi wanasoma kitabu.")]: a('noun_class',sw,0,en,tpl,g)
# --- object infixes ---
for sw,en,tpl,g in [
("Mama anafanya nini?","The mother loves the child.",f"{T('mother')} {V('3SG','PRES',0,'m','penda')} {T('child')}.","Mama anampenda mtoto."),
("Baba anafanya nini?","The father is helping the neighbour.",f"{T('father')} {V('3SG','PRES',0,'m','saidia')} {T('neighbour')}.","Baba anamsaidia jirani."),
("Dada anafanya nini?","The sister is teaching the children.",f"{T('sister')} {V('3SG','PRES',0,'wa','fundisha')} {T('children')}.","Dada anawafundisha watoto."),
("Rafiki yako alifanya nini?","The friend did not answer me.",f"{T('friend')} {V('3SG','PAST',1,'ni','jibu')}.","Rafiki hakunijibu."),
("Walikuambia nini?","They told us the news.",f"{V('3PL','PAST',0,'tu','ambia')} {T('news')}.","Walituambia habari."),
("Alikupa nini?","She gave me the book.",f"{V('3SG','PAST',0,'ni','pa')} {T('book')}.","Alinipa kitabu."),
("Mliniona lini?","We saw you yesterday.",f"{V('1PL','PAST',0,'ku','ona')} {T('yesterday')}.","Tulikuona jana."),
("Utaniita lini?","I will call you tomorrow.",f"{V('1SG','FUT',0,'ku','ita')} {T('tomorrow')}.","Nitakuita kesho."),
("Wanakupenda?","They love you.",f"{V('3PL','PRES',0,'ku','penda')}.","Wanakupenda."),
("Mnatupenda?","We love them.",f"{V('1PL','PRES',0,'wa','penda')}.","Tunawapenda.")]: a('object_infix',sw,0,en,tpl,g)
assert len(P)==100,len(P); assert len({p[1] for p in P})==100
C='/home/claude/sema_work/target/release/sema_cc'; D='data/terms.clinical.tsv,data/terms.core.tsv'
def run(*a):
    r=subprocess.run([C,*a],capture_output=True,text=True,cwd='/home/claude/sema_work'); 
    try: return json.loads(r.stdout)
    except Exception: return {'error':r.stderr.strip()}
def sc(s): 
    s=s.strip(); return s[:1].upper()+s[1:]
rows=[]
for i,(cat,sw,neg,en,tpl,gold) in enumerate(P,1):
    f=run('linearize','--sentence',sw)
    toks=f.get('tokens',[]); has_not=any(t['role']=='Negation' for t in toks)
    p=run('terms','--dict',D,'--text',en)
    slots=p.get('slots',[]); idtext=p.get('protected','')
    r=run('terms','--dict',D,'--template',tpl)
    out=sc(r['swahili']) if 'swahili' in r else 'ERROR: '+r.get('error','')
    rows.append(dict(id=f"P{i:03d}",cat=cat,swahili_in=sw,skeleton=f.get('telegram',''),unresolved=' '.join(f.get('unresolved',[])),input_negated=bool(neg),skeleton_has_NOT=has_not,
      model_reply_illustrative=en,locked=idtext,n_locked=len(slots),sema_output=out,gold=gold,exact=(out==gold)))
with open('pairs100.csv','w',newline='',encoding='utf-8') as f:
    w=csv.DictWriter(f,fieldnames=list(rows[0])); w.writeheader(); w.writerows(rows)
json.dump(P,open('pairs_src.json','w'),ensure_ascii=False)
N=len(rows); ex=sum(r['exact'] for r in rows)
neg=[r for r in rows if r['input_negated']]; 
print(f"STEP forward: negated inputs keeping NOT in skeleton {sum(r['skeleton_has_NOT'] for r in neg)}/{len(neg)}; sentences with an unresolved word: {sum(1 for r in rows if r['unresolved'])}/100")
print(f"STEP lock: replies with >=1 locked term/number: {sum(r['n_locked']>0 for r in rows)}/100; avg locked per reply {sum(r['n_locked'] for r in rows)/N:.1f}")
print(f"STEP render: exact match to gold {ex}/{N}")
by=collections.defaultdict(lambda:[0,0])
for r in rows: by[r['cat']][0]+=r['exact']; by[r['cat']][1]+=1
for k,v in by.items(): print(f"   {k:16s}{v[0]}/{v[1]}")
for r in rows:
    if not r['exact']: print('FAIL',r['id'],'|',r['sema_output'],'| gold:',r['gold'])
for r in neg:
    if not r['skeleton_has_NOT']: print('NO-NOT',r['swahili_in'],'->',r['skeleton'])
for r in rows:
    if r['unresolved']: print('UNRES',r['swahili_in'],r['unresolved'])
