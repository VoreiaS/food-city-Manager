const RESTAURANTS = [
  {id:"00000000-0000-1000-0000-000000000001",name:"Spice Villa",slug:"spice-villa",description:"Authentic Indian cuisine with a modern twist. Famous for our biryanis and tandoori.",cuisine_types:["indian","vegetarian"],price_range:2,logo_url:null,cover_url:null,lat:6.9271,lng:79.8612,delivery_radius_m:5000,delivery_fee_cents:250,min_order_cents:1000,status:"active",rating_avg:4.6,rating_count:1243,is_open:true},
  {id:"00000000-0000-1000-0000-000000000002",name:"Pizza Hub",slug:"pizza-hub",description:"Wood-fired Neapolitan pizzas, fresh pasta, and Italian antipasti.",cuisine_types:["italian","pizza"],price_range:2,logo_url:null,cover_url:null,lat:6.9275,lng:79.8618,delivery_radius_m:6000,delivery_fee_cents:300,min_order_cents:1500,status:"active",rating_avg:4.4,rating_count:892,is_open:true},
  {id:"00000000-0000-1000-0000-000000000003",name:"Sushi World",slug:"sushi-world",description:"Omakase-style sushi, ramen, and Japanese small plates.",cuisine_types:["japanese","sushi"],price_range:3,logo_url:null,cover_url:null,lat:6.9280,lng:79.8605,delivery_radius_m:4500,delivery_fee_cents:350,min_order_cents:2000,status:"active",rating_avg:4.8,rating_count:567,is_open:true}
];
const MENUS = {
  "00000000-0000-1000-0000-000000000001":{restaurant_id:"00000000-0000-1000-0000-000000000001",menu_version:1,categories:[
    {id:"cat-1",name:"Starters",sort_order:1,items:[
      {id:"item-1",name:"Paneer Tikka",description:"Cottage cheese marinated in spiced yogurt, char-grilled.",price_cents:850,image_url:null,is_veg:true,is_vegan:false,is_halal:false,spice_level:2,allergens:["dairy"],in_stock:true,sort_order:1,customizations:[]},
      {id:"item-2",name:"Samosa (2 pc)",description:"Crispy pastry filled with spiced potatoes and peas.",price_cents:450,image_url:null,is_veg:true,is_vegan:false,is_halal:false,spice_level:1,allergens:["gluten"],in_stock:true,sort_order:2,customizations:[]},
      {id:"item-3",name:"Chicken 65",description:"Deep-fried chicken with curry leaves and red chilies.",price_cents:950,image_url:null,is_veg:false,is_vegan:false,is_halal:false,spice_level:3,allergens:[],in_stock:true,sort_order:3,customizations:[]}
    ]},
    {id:"cat-2",name:"Mains",sort_order:2,items:[
      {id:"item-4",name:"Butter Chicken",description:"Tandoori chicken simmered in creamy tomato gravy.",price_cents:1450,image_url:null,is_veg:false,is_vegan:false,is_halal:false,spice_level:1,allergens:["dairy"],in_stock:true,sort_order:1,customizations:[]},
      {id:"item-5",name:"Paneer Makhani",description:"Cottage cheese in a rich tomato-cashew gravy.",price_cents:1350,image_url:null,is_veg:true,is_vegan:false,is_halal:false,spice_level:1,allergens:["dairy","nuts"],in_stock:true,sort_order:2,customizations:[]},
      {id:"item-6",name:"Dal Makhani",description:"Black lentils slow-cooked overnight with butter.",price_cents:1100,image_url:null,is_veg:true,is_vegan:false,is_halal:false,spice_level:0,allergens:["dairy"],in_stock:true,sort_order:3,customizations:[]}
    ]},
    {id:"cat-3",name:"Breads & Rice",sort_order:3,items:[
      {id:"item-7",name:"Hyderabadi Biryani",description:"Long-grain basmati layered with spiced meat, sealed and dum-cooked.",price_cents:1650,image_url:null,is_veg:false,is_vegan:false,is_halal:false,spice_level:2,allergens:[],in_stock:true,sort_order:1,customizations:[]},
      {id:"item-8",name:"Veg Biryani",description:"Fragrant basmati with mixed vegetables and saffron.",price_cents:1350,image_url:null,is_veg:true,is_vegan:false,is_halal:false,spice_level:2,allergens:[],in_stock:true,sort_order:2,customizations:[]},
      {id:"item-9",name:"Garlic Naan",description:"Tandoor-baked flatbread brushed with garlic butter.",price_cents:250,image_url:null,is_veg:true,is_vegan:false,is_halal:false,spice_level:0,allergens:["gluten","dairy"],in_stock:true,sort_order:3,customizations:[]}
    ]},
    {id:"cat-4",name:"Desserts",sort_order:4,items:[
      {id:"item-10",name:"Gulab Jamun (2 pc)",description:"Warm milk dumplings in rose-scented syrup.",price_cents:350,image_url:null,is_veg:true,is_vegan:false,is_halal:false,spice_level:0,allergens:["dairy"],in_stock:true,sort_order:1,customizations:[]}
    ]}
  ]},
  "00000000-0000-1000-0000-000000000002":{restaurant_id:"00000000-0000-1000-0000-000000000002",menu_version:1,categories:[
    {id:"cat-5",name:"Pizzas",sort_order:1,items:[
      {id:"item-11",name:"Margherita",description:"San Marzano tomato, fresh mozzarella, basil.",price_cents:1200,image_url:null,is_veg:true,is_vegan:false,is_halal:false,spice_level:0,allergens:["gluten","dairy"],in_stock:true,sort_order:1,customizations:[]},
      {id:"item-12",name:"Pepperoni",description:"Tomato, mozzarella, double pepperoni.",price_cents:1550,image_url:null,is_veg:false,is_vegan:false,is_halal:false,spice_level:0,allergens:["gluten","dairy"],in_stock:true,sort_order:2,customizations:[]},
      {id:"item-13",name:"Quattro Formaggi",description:"Mozzarella, gorgonzola, fontina, parmesan.",price_cents:1750,image_url:null,is_veg:true,is_vegan:false,is_halal:false,spice_level:0,allergens:["gluten","dairy"],in_stock:true,sort_order:3,customizations:[]}
    ]},
    {id:"cat-6",name:"Pasta",sort_order:2,items:[
      {id:"item-14",name:"Spaghetti Carbonara",description:"Pancetta, egg, pecorino, black pepper.",price_cents:1450,image_url:null,is_veg:false,is_vegan:false,is_halal:false,spice_level:0,allergens:["gluten","egg","dairy"],in_stock:true,sort_order:1,customizations:[]},
      {id:"item-15",name:"Penne Arrabbiata",description:"Tomato, garlic, chili, parsley.",price_cents:1250,image_url:null,is_veg:true,is_vegan:false,is_halal:false,spice_level:2,allergens:["gluten"],in_stock:true,sort_order:2,customizations:[]}
    ]}
  ]},
  "00000000-0000-1000-0000-000000000003":{restaurant_id:"00000000-0000-1000-0000-000000000003",menu_version:1,categories:[
    {id:"cat-7",name:"Nigiri & Sashimi",sort_order:1,items:[
      {id:"item-18",name:"Salmon Nigiri (2 pc)",description:"Fresh Atlantic salmon over seasoned rice.",price_cents:600,image_url:null,is_veg:false,is_vegan:false,is_halal:false,spice_level:0,allergens:["fish"],in_stock:true,sort_order:1,customizations:[]},
      {id:"item-19",name:"Tuna Sashimi (5 pc)",description:"Sliced bluefin tuna, no rice.",price_cents:950,image_url:null,is_veg:false,is_vegan:false,is_halal:false,spice_level:0,allergens:["fish"],in_stock:true,sort_order:2,customizations:[]}
    ]},
    {id:"cat-8",name:"Rolls",sort_order:2,items:[
      {id:"item-20",name:"California Roll (8 pc)",description:"Crab, avocado, cucumber, sesame.",price_cents:850,image_url:null,is_veg:false,is_vegan:false,is_halal:false,spice_level:0,allergens:["fish","sesame"],in_stock:true,sort_order:1,customizations:[]},
      {id:"item-21",name:"Spicy Tuna Roll (8 pc)",description:"Tuna, spicy mayo, scallions.",price_cents:950,image_url:null,is_veg:false,is_vegan:false,is_halal:false,spice_level:2,allergens:["fish","egg"],in_stock:true,sort_order:2,customizations:[]}
    ]},
    {id:"cat-9",name:"Ramen",sort_order:3,items:[
      {id:"item-23",name:"Tonkotsu Ramen",description:"Pork bone broth, chashu, egg, scallions, noodles.",price_cents:1650,image_url:null,is_veg:false,is_vegan:false,is_halal:false,spice_level:1,allergens:["gluten","egg","soy"],in_stock:true,sort_order:1,customizations:[]},
      {id:"item-24",name:"Miso Vegetable Ramen",description:"Miso broth, tofu, vegetables, noodles.",price_cents:1450,image_url:null,is_veg:true,is_vegan:false,is_halal:false,spice_level:0,allergens:["gluten","soy"],in_stock:true,sort_order:2,customizations:[]}
    ]}
  ]}
};

function tok(e,r){return "mock."+Buffer.from(JSON.stringify({sub:"mock-user-id",email:e,role:r||"customer",token_type:"access",exp:Math.floor(Date.now()/1000)+900})).toString("base64").replace(/=/g,"")}

module.exports = async function handler(req, res) {
  const path = (req.url||"").replace(/^\/api\/v1/,"").split("?")[0];
  const m = req.method;
  let b={}; if(m==="POST"||m==="PATCH"){try{b=typeof req.body==="string"?JSON.parse(req.body):(req.body||{})}catch{b={}}}

  if(path==="/health"||path==="/ready")return res.status(200).send("ok");
  if(path==="/auth/register"&&m==="POST")return res.status(201).json({user:{id:"mock-user-id",email:b.email,phone:b.phone,full_name:b.full_name,role:b.role||"customer",created_at:new Date().toISOString()},access_token:tok(b.email,b.role),refresh_token:"mock-refresh",expires_in:900});
  if(path==="/auth/login"&&m==="POST")return res.status(200).json({user:{id:"mock-user-id",email:b.email,phone:"+15551234567",full_name:"Demo User",role:"customer",created_at:new Date().toISOString()},access_token:tok(b.email),refresh_token:"mock-refresh",expires_in:900});
  if(path==="/auth/refresh"&&m==="POST")return res.status(200).json({user:{id:"mock-user-id",email:"user@example.com",phone:"+15551234567",full_name:"Demo User",role:"customer",created_at:new Date().toISOString()},access_token:tok("user@example.com"),refresh_token:"mock-refresh",expires_in:900});
  if(path==="/auth/me"&&m==="GET")return res.status(200).json({id:"mock-user-id",email:"user@example.com",phone:"+15551234567",full_name:"Demo User",role:"customer",created_at:new Date().toISOString()});
  if(path==="/restaurants"&&m==="GET"){const c=RESTAURANTS.map(r=>({...r,distance_m:Math.floor(Math.random()*2000),delivery_eta_min:25+Math.floor(Math.random()*15)}));return res.status(200).json({data:c,page:1,page_size:30,total:c.length})}
  if(path==="/restaurants/cuisines"&&m==="GET")return res.status(200).json([...new Set(RESTAURANTS.flatMap(r=>r.cuisine_types))]);
  const rm=path.match(/^\/restaurants\/([0-9a-f-]+)$/);if(rm&&m==="GET"){const r=RESTAURANTS.find(x=>x.id===rm[1]||x.slug===rm[1]);if(!r)return res.status(404).json({error:{code:"not_found",message:"restaurant not found"}});return res.status(200).json(r)}
  const mm=path.match(/^\/restaurants\/([0-9a-f-]+)\/menu$/);if(mm&&m==="GET"){const menu=MENUS[mm[1]];if(!menu)return res.status(404).json({error:{code:"not_found",message:"menu not found"}});return res.status(200).json(menu)}
  if(path==="/addresses"&&m==="GET")return res.status(200).json([{id:"addr-1",user_id:"mock-user-id",label:"Home",line1:"123 Main St",line2:null,city:"Colombo",postal_code:null,lat:6.9271,lng:79.8612,formatted_address:"123 Main St, Colombo",is_default:true}]);
  if(path==="/addresses"&&m==="POST")return res.status(201).json({id:"addr-"+Date.now(),user_id:"mock-user-id",label:b.label,line1:b.line1,line2:b.line2||null,city:b.city,postal_code:b.postal_code||null,lat:b.lat,lng:b.lng,formatted_address:b.formatted_address,is_default:b.is_default||false});
  if(path==="/cart"&&m==="GET")return res.status(200).json(null);
  if(path==="/cart/items"&&m==="POST"){const rest=RESTAURANTS.find(r=>r.id===b.restaurant_id);const menu=MENUS[b.restaurant_id];const items=menu?menu.categories.flatMap(c=>c.items):[];const item=items.find(i=>i.id===b.menu_item_id);if(!item)return res.status(404).json({error:{code:"not_found",message:"item not found"}});const ci={id:"ci-"+Date.now(),cart_id:"cart-1",menu_item_id:item.id,menu_item_name:item.name,menu_item_image_url:item.image_url,base_price_cents:item.price_cents,quantity:b.quantity,customizations:[],notes:b.notes||null,line_total_cents:item.price_cents*b.quantity};const sub=ci.line_total_cents;const df=rest?.delivery_fee_cents||0;return res.status(200).json({id:"cart-1",user_id:"mock-user-id",restaurant_id:b.restaurant_id,restaurant_name:rest?.name||"Restaurant",status:"active",items:[ci],subtotal_cents:sub,delivery_fee_cents:df,total_cents:sub+df,min_order_cents:rest?.min_order_cents||0,meets_min_order:sub>=(rest?.min_order_cents||0)})}
  if(path==="/orders"&&m==="GET")return res.status(200).json([]);
  if(path==="/orders"&&m==="POST"){const oid="order-"+Date.now();return res.status(201).json({order:{id:oid,customer_id:"mock-user-id",restaurant_id:"00000000-0000-1000-0000-000000000001",driver_id:null,status:"pending_accept",payment_status:"succeeded",snapshot:{},subtotal_cents:2900,delivery_fee_cents:250,tax_cents:0,tip_cents:b.tip_cents||0,discount_cents:0,total_cents:2900+250+(b.tip_cents||0),currency:"usd",delivery_address:{},notes:b.notes||null,placed_at:new Date().toISOString(),accepted_at:null,preparing_at:null,ready_at:null,picked_up_at:null,delivered_at:null,canceled_at:null,cancellation_reason:null,estimated_delivery_at:new Date(Date.now()+45*60000).toISOString(),created_at:new Date().toISOString(),updated_at:new Date().toISOString(),items:[{id:"oi-1",menu_item_id:"item-4",name:"Butter Chicken",description:null,price_cents:1450,quantity:2,customizations:[],notes:null,status:"pending",line_total_cents:2900}],restaurant_name:"Spice Villa"},payment:{intent_id:"pi-mock",provider_intent_id:"mock_pi",client_secret:"mock_secret",status:"succeeded",amount_cents:2900+250+(b.tip_cents||0),currency:"usd",mock_mode:true}})}
  if(path==="/loyalty/me"&&m==="GET")return res.status(200).json({points_balance:1250,tier:"gold",lifetime_points:5420,next_tier_points:10000,tier_benefits:["Earn 1 point per $1","Free delivery"]});
  if(path==="/loyalty/me/transactions"&&m==="GET")return res.status(200).json([{id:"lt-1",account_id:"la-1",points_delta:29,reason:"order_delivered",order_id:"order-1",created_at:new Date().toISOString()}]);
  return res.status(404).json({error:{code:"not_found",message:`Route ${m} ${path} not found`}});
}
