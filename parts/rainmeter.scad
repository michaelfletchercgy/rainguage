// All sizes are in mm
diameter = 100;
funnelheight = diameter*0.6;
assemblyheight = diameter;

clapperheight = 0.5*(assemblyheight-funnelheight);
clapperwidth = 0.6*(assemblyheight-funnelheight);
clapperlength = 0.8*diameter;
minordiameter = diameter/10;
thick = 2;
fudge = 0.5;

// set below to true for print layout, something else for assembly
printing = true;

square_centimeters =  diameter * diameter / 4 * 3.1415926 / 100;

echo( "Funnel surface = ",square_centimeters,"cm²");
echo( "Adding 100 milliliter of water is thus ", 1000 / square_centimeters, "mm precipitation");


if( printing )
{
	translate([.7*diameter,0,0])
		funnel();
	translate([-.7*diameter,0,0])
		base();
	clapper();
}
else
{
	color([1,1,0])
			base();
	color([1,0,1])
	difference() {
		translate([0,0,assemblyheight+thick])
		rotate([180,0,0])
			funnel();
		translate([thick, thick, 2*thick])
			cube([diameter, diameter, diameter]);
	}
	color([0,1,1])
	translate([0,0,clapperheight-2])
	rotate([30,0,45])
	translate([0,0,-2])
		clapper();
}

module funnel()
{
	union() {
		// Side walls
		difference() {
			cylinder(r=diameter/2, h=assemblyheight);
			translate([0,0,-thick])
			 	cylinder(r=diameter/2-thick, h=assemblyheight+2*thick);
			// mounting holes
			translate([0,0, assemblyheight-4*thick+fudge])
			{
				rotate([0,90,0])
						cylinder(r=1.5,h=diameter,center=true);
				rotate([90,90,0])
						cylinder(r=1.5,h=diameter,center=true);
			}
		}
		// Funnel
		difference() {
			cylinder(r1=diameter/2, r2=0, h=funnelheight);
			translate([0,0,-thick*1.0])
				cylinder(r1=diameter/2, r2=0, h=funnelheight);
			cylinder(r=minordiameter/6, h=assemblyheight);
			// Funnel holes
			for(i=[0:60:360])
				rotate([0,0,i])
				translate([minordiameter/2,0,0])
					cylinder(r=minordiameter/5, h=assemblyheight);
			
		}

	}
}


module base()
{	
	union() {
		// Cross base
		translate([0,0,thick/2])
		{
			cube([diameter,minordiameter,thick], center=true);
			cube([minordiameter,diameter,thick], center=true);
		}
		// Clapper mounting
		rotate([0,0,45])
		{
			difference()
			{	
				union() 
				{
					translate([clapperwidth/2+fudge,-diameter/8])
						cube([thick, diameter/4,clapperheight]);	
					translate([-clapperwidth/2-fudge-thick,-diameter/8])
						cube([thick, diameter/4,clapperheight]);
				}	
				translate([0,0,clapperheight-2])
					rotate([0,90,0])
						cylinder(r=1.5,h=diameter,center=true);
			}
		}
		// Funnel mounting 
		difference() {
			cylinder(r=diameter/2-thick-fudge, h=clapperheight);
			translate([0,0,-thick])
			 	cylinder(r=diameter/2-thick*2, h=clapperheight+2*thick);
			translate([0,0,5*thick])
			{
				rotate([0,90,0])
						cylinder(r=1.5,h=diameter,center=true);
				rotate([90,90,0])
						cylinder(r=1.5,h=diameter,center=true);
			}
		}
	}
}


module clapper()
{
	intersection()
	{
		difference()
		{
			translate([-clapperwidth/2,0,0]) 
			{
				union() 
				{
					//Sides
					translate([clapperwidth-thick,0,0])
					{
						vertTriangle(clapperlength,clapperheight,thick);
					} 
					vertTriangle(clapperlength,clapperheight,thick);
					//Base
					translate([0,-clapperlength/2,0])
						cube([clapperwidth,clapperlength,thick]);
					//Center
					vertTriangle(clapperlength/6,clapperheight,clapperwidth);
				}
			}
			// Mounting hole
			translate([0,0,2])
				rotate([0,90,0])
					cylinder(r=1.5,h=diameter,center=true);
		}
		// Taper ends
		rotate([0,0,45])
			scale(0.8)
			cube([clapperlength,clapperlength,clapperlength], center=true);
	}
}

module vertTriangle(base,height,thick)
{
	rotate([90,00,90])
	linear_extrude(height=thick)
	{
		polygon(points=[[-base/2,0],[base/2,0],[0,height]], paths=[[0,1,2]]);
	}
}
